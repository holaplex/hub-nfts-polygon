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
    futures_util::future::try_join_all,
    prelude::anyhow,
    producer::Producer,
    prost_types::Timestamp,
    serde_json,
};
use poem::{handler, web::Data, Request, Result};

use crate::{types::*, PayloadBytes, Signature, NULL_ADDRESS};

#[handler]
pub async fn process(
    payload: PayloadBytes,
    signature: Signature,
    _req: &Request,
    processor: Data<&NftActivityController>,
) -> Result<()> {
    let Data(processor) = processor;

    processor.process_payload(signature, payload).await
}

#[derive(Clone)]
pub struct NftActivityController {
    db: Connection,
    producer: Producer<PolygonNftEvents>,
    deployer_addr: String,
    signing_key: Vec<u8>,
}

impl NftActivityController {
    pub fn new(
        db: Connection,
        producer: Producer<PolygonNftEvents>,
        deployer_addr: String,
        signing_key: Vec<u8>,
    ) -> Self {
        Self {
            db,
            producer,
            deployer_addr,
            signing_key,
        }
    }

    pub async fn process_payload(&self, signature: Signature, bytes: PayloadBytes) -> Result<()> {
        let payload: Payload =
            serde_json::from_slice(&bytes.0).context("failed to parse payload")?;

        let ts = Timestamp::from_str(&payload.created_at).context("failed to parse timestamp")?;

        bytes.verify(&signature, &self.signing_key)?;

        if payload.ty == EventType::NftActivity {
            for event in payload.event.activity {
                self.process_nft_activity(event, &ts).await?;
            }
        }
        Ok(())
    }

    async fn process_nft_activity(&self, event: ActivityPayload, ts: &Timestamp) -> Result<()> {
        if event.from_address == self.deployer_addr || event.from_address == NULL_ADDRESS {
            return Ok(());
        }

        let erc1155_tokens = event
            .clone()
            .erc1155_metadata
            .context("Erc1155 Metadata not found")?;

        let futures = erc1155_tokens
            .into_iter()
            .map(|token| {
                let db = self.db.clone();
                let ts = ts.clone();
                let event = event.clone();
                let self_cloned = self.clone();

                tokio::spawn(async move {
                    let edition_id = strip_prefix(token.token_id)?;
                    let value = strip_prefix(token.value)?;

                    let mints =
                        Mint::get_mints_for_edition(&db, &event.from_address, edition_id, value)
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

                    self_cloned
                        .update_mints_owner(&mints, &event.to_address)
                        .await?;
                    self_cloned
                        .emit_event(&mints, &event.to_address, &ts, &event.hash)
                        .await
                })
            })
            .collect::<Vec<_>>();

        try_join_all(futures)
            .await
            .context("failed to process all futures")?;

        Ok(())
    }

    async fn emit_event(
        &self,
        mints: &[mints::Model],
        new_owner: &str,
        ts: &Timestamp,
        hash: &str,
    ) -> Result<()> {
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
                transaction_hash: hash.to_string(),
            })),
        };

        self.producer
            .send(Some(&event), Some(&key))
            .await
            .context("failed to emit event")?;

        Ok(())
    }

    async fn update_mints_owner(
        &self,
        mints: &[mints::Model],
        new_owner: &str,
    ) -> anyhow::Result<()> {
        let txn = self.db.get().begin().await?;

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
}

fn strip_prefix(s: String) -> Result<u64> {
    let without_prefix = s.strip_prefix("0x").context("No 0x prefix found")?;

    let val = u64::from_str_radix(without_prefix, 16).context("Failed to parse to u64")?;
    Ok(val)
}
