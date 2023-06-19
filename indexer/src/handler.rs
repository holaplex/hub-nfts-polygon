use std::str::FromStr;

use holaplex_hub_nfts_polygon_core::{
    db::Connection,
    proto::{
        polygon_nft_events::Event, MintedTokensOwnershipUpdate, PolygonNftEventKey,
        PolygonNftEvents,
    },
    sea_orm::{ActiveModelTrait, Set, TransactionTrait},
    Mint,
};
use holaplex_hub_nfts_polygon_entity::mints;
use hub_core::{
    anyhow::{self, Context},
    prelude::anyhow,
    producer::Producer,
    prost_types::Timestamp,
};
use poem::{handler, web::Data, Request, Result};

use crate::{types::*, Signature};

#[handler]
pub async fn process(
    payload: Payload,
    signature: Signature,
    _req: &Request,
    db: Data<&Connection>,
    producer: Data<&Producer<PolygonNftEvents>>,
    signing_key: Data<&Vec<u8>>,
) -> Result<()> {
    let Data(db) = db;
    let Data(producer) = producer;

    let ts = Timestamp::from_str(&payload.created_at).context("failed to parse timestamp")?;

    payload.verify(&signature, signing_key.0)?;

    if payload.ty == EventType::NftActivity {
        for event in payload.event.activity {
            process_nft_activity(db, producer, event, &ts).await?;
        }
    };

    Ok(())
}

async fn process_nft_activity(
    db: &Connection,
    producer: &Producer<PolygonNftEvents>,
    event: ActivityPayload,
    ts: &Timestamp,
) -> Result<()> {
    let erc1155_tokens = event
        .erc1155_metadata
        .context("Erc1155 Metadata not found")?;

    for token in erc1155_tokens {
        let edition_id = strip_prefix(token.token_id)?;
        let value = strip_prefix(token.value)?;

        let mints = Mint::get_mints_for_edition(db, &event.from_address, edition_id, value)
            .await
            .context("failed to get mints")?;

        if mints.len() != value as usize {
            return Err(anyhow!(
                "Expected {} mints for edition {}, but found {}",
                value,
                edition_id,
                mints.len()
            )
            .into());
        }

        update_mints_owner(db, &mints, &event.to_address).await?;
        emit_event(producer, &mints, &event.to_address, ts).await?;
    }
    Ok(())
}

async fn emit_event(
    producer: &Producer<PolygonNftEvents>,
    mints: &[mints::Model],
    new_owner: &str,
    ts: &Timestamp,
) -> anyhow::Result<()> {
    let collection_id = mints
        .get(0)
        .context("No mints found")?
        .collection_id
        .to_string();
    let mint_ids = mints
        .iter()
        .map(|m| m.id.to_string())
        .collect::<Vec<String>>();

    let key = PolygonNftEventKey {
        id: collection_id,
        user_id: String::new(),
        project_id: String::new(),
    };

    let event = PolygonNftEvents {
        event: Some(Event::UpdateMintsOwner(MintedTokensOwnershipUpdate {
            mint_ids,
            new_owner: new_owner.to_string(),
            timestamp: Some(ts.clone()),
        })),
    };

    producer.send(Some(&event), Some(&key)).await?;

    Ok(())
}

async fn update_mints_owner(
    db: &Connection,
    mints: &[mints::Model],
    new_owner: &str,
) -> anyhow::Result<()> {
    let txn = db.get().begin().await?;

    for mint in mints {
        let mut mint_am: mints::ActiveModel = mint.clone().into();
        mint_am.owner = Set(new_owner.to_string());
        mint_am
            .update(&txn)
            .await
            .context("failed to update mint")?;
    }

    txn.commit()
        .await
        .map_err(|e| anyhow!(format!("failed to update mints {e}")))
}

fn strip_prefix(s: String) -> Result<u64> {
    let without_prefix = s.strip_prefix("0x").context("No 0x prefix found")?;

    let val = u64::from_str_radix(without_prefix, 16).context("Failed to parse to u64")?;
    Ok(val)
}
