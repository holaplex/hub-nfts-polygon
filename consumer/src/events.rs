use ethers::types::{Bytes, U256};
use holaplex_hub_nfts_polygon_core::{
    db::Connection,
    proto::{
        self,
        polygon_events::Event as PolygonEvents,
        polygon_nft_events,
        treasury_events::{EcdsaSignature, Event as TreasuryEvents, PolygonPermitHashSignature},
        CreateEditionTransaction, MintEditionTransaction, NftEventKey, PermitArgsHash,
        PolygonNftEventKey, PolygonNftEvents, PolygonTokenTransferTxns, PolygonTransaction,
        TransferPolygonAsset, TreasuryEventKey, UpdateEdtionTransaction,
    },
    sea_orm::Set,
    Collection, EditionInfo, Mint, Services,
};
use holaplex_hub_nfts_polygon_entity::{collections, mints};
use hub_core::{chrono::Utc, prelude::*, producer::Producer, uuid::Uuid};

use crate::EditionContract;

#[derive(Clone)]
pub struct Processor {
    db: Connection,
    producer: Producer<PolygonNftEvents>,
    edition_contract: EditionContract,
}

impl Processor {
    #[must_use]
    pub fn new(
        db: Connection,
        producer: Producer<PolygonNftEvents>,
        edition_contract: EditionContract,
    ) -> Self {
        Self {
            db,
            producer,
            edition_contract,
        }
    }

    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn process(&self, msg: Services) -> Result<()> {
        // match topics
        match msg {
            Services::Nfts(key, e) => match e.event {
                Some(PolygonEvents::CreateDrop(payload)) => {
                    self.create_polygon_edition(key, payload).await
                },
                Some(PolygonEvents::RetryDrop(payload)) => self.retry_drop(key, payload).await,
                Some(PolygonEvents::MintDrop(payload)) => self.mint_drop(key, payload).await,
                Some(PolygonEvents::UpdateDrop(payload)) => self.update_drop(key, payload).await,
                Some(PolygonEvents::RetryMintDrop(payload)) => self.retry_mint(key, payload).await,
                Some(PolygonEvents::TransferAsset(payload)) => {
                    self.sign_permit_token_transfer_hash(key, payload).await
                },
                None => Ok(()),
            },
            Services::Treasuries(key, e) => match e.event {
                Some(TreasuryEvents::PolygonPermitTransferTokenHashSigned(p)) => {
                    self.send_transfer_asset_txns(key, p).await
                },
                Some(_) | None => Ok(()),
            },
        }
    }

    async fn create_polygon_edition(
        &self,
        key: NftEventKey,
        payload: CreateEditionTransaction,
    ) -> Result<()> {
        let CreateEditionTransaction {
            edition_info,
            fee_receiver,
            fee_numerator,
            amount,
            ..
        } = payload;

        let key = PolygonNftEventKey {
            id: key.id,
            user_id: key.user_id,
            project_id: key.project_id,
        };

        let edition_info: proto::EditionInfo =
            edition_info.context("EditionInfo not found in event payload")?;
        let edition_id = Collection::find_max_edition_id(&self.db)
            .await?
            .unwrap_or(0)
            + 1;

        let deployer = self.edition_contract.owner().await?;
        let owner = format!("{deployer:?}");

        Collection::create(&self.db, collections::Model {
            id: Uuid::from_str(&key.id)?,
            edition_id,
            fee_receiver: fee_receiver.clone(),
            owner,
            creator: edition_info.creator.clone(),
            uri: edition_info.uri.clone(),
            name: edition_info.collection.clone(),
            description: edition_info.description.clone(),
            image_uri: edition_info.image_uri.clone(),
            created_at: Utc::now().naive_utc(),
        })
        .await?;

        let typed_tx = self
            .edition_contract
            .create_edition(
                edition_id.into(),
                edition_info.try_into()?,
                deployer,
                amount.into(),
                fee_receiver.parse()?,
                fee_numerator.try_into()?,
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitCreateDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id,
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key)).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }

    async fn retry_drop(&self, key: NftEventKey, payload: CreateEditionTransaction) -> Result<()> {
        let CreateEditionTransaction {
            fee_receiver,
            fee_numerator,
            receiver,
            amount,
            ..
        } = payload;

        let collection = Collection::find_by_id(&self.db, key.id.parse()?)
            .await?
            .context(format!("No collection found for id {:?}", key.id))?;

        let edition_info = EditionInfo {
            description: collection.description,
            image_uri: collection.image_uri,
            collection: collection.name,
            uri: collection.uri,
            creator: collection.creator.parse()?,
        };

        let typed_tx = self
            .edition_contract
            .create_edition(
                collection.edition_id.into(),
                edition_info,
                receiver.parse()?,
                amount.into(),
                fee_receiver.parse()?,
                fee_numerator.try_into()?,
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitRetryCreateDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id: collection.edition_id,
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key.into())).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }

    async fn retry_mint(&self, key: NftEventKey, payload: MintEditionTransaction) -> Result<()> {
        let MintEditionTransaction {
            receiver,
            amount,
            collection_id,
        } = payload;

        let collection = Collection::find_by_id(&self.db, collection_id.parse()?)
            .await?
            .context("collection not found")?;

        Mint::find_by_id(&self.db, key.id.parse()?)
            .await?
            .context("mint not found")?;

        let deployer = self.edition_contract.owner().await?;

        let typed_tx = self
            .edition_contract
            .safe_transfer_from(
                deployer,
                receiver.parse()?,
                collection.edition_id.into(),
                amount.into(),
                Bytes::new(),
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitRetryMintDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id: collection.edition_id,
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key.into())).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }

    async fn mint_drop(&self, key: NftEventKey, payload: MintEditionTransaction) -> Result<()> {
        let MintEditionTransaction {
            receiver,
            amount,
            collection_id,
        } = payload;

        let collection = Collection::find_by_id(&self.db, collection_id.parse()?)
            .await?
            .context(format!("No collection found for id {:?}", key.id))?;
        Mint::create(&self.db, mints::Model {
            id: key.id.parse()?,
            collection_id: collection.id,
            owner: receiver.parse()?,
            amount: amount.try_into()?,
            created_at: Utc::now().naive_utc(),
        })
        .await?;

        let typed_tx = self
            .edition_contract
            .safe_transfer_from(
                collection.owner.parse()?,
                receiver.parse()?,
                collection.edition_id.into(),
                amount.into(),
                Bytes::new(),
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitMintDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id: collection.edition_id,
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key.into())).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }

    async fn update_drop(&self, key: NftEventKey, payload: UpdateEdtionTransaction) -> Result<()> {
        let UpdateEdtionTransaction { edition_info } = payload;

        let edition_info = edition_info.context("EditionInfo not found in event payload")?;
        let proto::EditionInfo {
            description,
            image_uri,
            uri,
            creator,
            collection,
            ..
        } = edition_info.clone();

        let collection_model = Collection::find_by_id(&self.db, key.id.parse()?)
            .await?
            .context("collection not found")?;

        let mut collection_am = Collection::get_active_model(collection_model.clone());
        collection_am.description = Set(description);
        collection_am.name = Set(collection);
        collection_am.image_uri = Set(image_uri);
        collection_am.uri = Set(uri);
        collection_am.creator = Set(creator);
        Collection::update(&self.db, collection_am).await?;

        let typed_tx = self
            .edition_contract
            .edit_edition(collection_model.edition_id.into(), edition_info.try_into()?)
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitUpdateDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id: collection_model.edition_id,
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key.into())).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }

    async fn sign_permit_token_transfer_hash(
        &self,
        key: NftEventKey,
        payload: TransferPolygonAsset,
    ) -> Result<()> {
        let TransferPolygonAsset {
            collection_mint_id,
            owner_address,
            recipient_address,
            amount,
        } = payload;

        let (_, collection) = Mint::find_with_collection(&self.db, collection_mint_id.parse()?)
            .await?
            .context(format!("No mint found for id {collection_mint_id}"))?;

        let collection = collection.context("No collection found")?;

        let hash = self
            .edition_contract
            .get_hash_typed_data_v4(
                owner_address.parse()?,
                collection.owner.parse()?,
                collection.edition_id.into(),
                amount.into(),
                U256::MAX,
            )
            .await
            .context("failed to get hash of the data")?;

        let event = PolygonNftEvents {
            event: Some(polygon_nft_events::Event::SignPermitTokenTransferHash(
                PermitArgsHash {
                    data: hash.to_vec(),
                    owner: owner_address,
                    spender: collection.owner,
                    recipient: recipient_address,
                    edition_id: collection.edition_id,
                    amount,
                },
            )),
        };

        self.producer.send(Some(&event), Some(&key.into())).await?;

        Ok(())
    }

    async fn send_transfer_asset_txns(
        &self,
        key: TreasuryEventKey,
        payload: PolygonPermitHashSignature,
    ) -> Result<()> {
        let PolygonPermitHashSignature {
            signature,
            owner,
            spender,
            recipient,
            edition_id,
            amount,
        } = payload;

        let EcdsaSignature { r, s, v } = signature.context("No ECDSA Signature found")?;

        let permit_tx = self
            .edition_contract
            .permit(
                owner.parse()?,
                spender.parse()?,
                edition_id.into(),
                amount.into(),
                U256::MAX,
                v.try_into()?,
                r.try_into()
                    .map_err(|_| anyhow!("failed to parse r component of ECDSA"))?,
                s.try_into()
                    .map_err(|_| anyhow!("failed to parse s component of ECDSA"))?,
            )
            .tx;

        let safe_transfer_from = self
            .edition_contract
            .safe_transfer_from(
                spender.parse()?,
                recipient.parse()?,
                edition_id.into(),
                amount.into(),
                Bytes::new(),
            )
            .tx;

        let permit_tx_data = permit_tx.data().context("No data in permit tx")?;
        let safe_transfer_from_data = safe_transfer_from
            .data()
            .context("No data in safe transfer from tx")?;

        let event = PolygonNftEvents {
            event: Some(polygon_nft_events::Event::SubmitTransferAssetTxns(
                PolygonTokenTransferTxns {
                    permit_token_transfer_txn: Some(PolygonTransaction {
                        data: permit_tx_data.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id,
                    }),
                    safe_transfer_from_txn: Some(PolygonTransaction {
                        data: safe_transfer_from_data.0.to_vec(),
                        contract_address: self.edition_contract.address().to_string(),
                        edition_id,
                    }),
                },
            )),
        };

        self.producer.send(Some(&event), Some(&key.into())).await?;

        Ok(())
    }
}
