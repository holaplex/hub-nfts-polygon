use holaplex_hub_nfts_polygon_core::{db::Connection, sea_orm::Set, Collection};
use holaplex_hub_nfts_polygon_entity::collections;
use hub_core::{anyhow::Error, chrono::Utc, prelude::*, producer::Producer, uuid::Uuid};

use crate::{
    edition_contract,
    proto::{
        self, polygon_events::Event as PolygonEvents, polygon_nft_events, CreateEditionTransaction,
        MintEditionTransaction, NftEventKey, PolygonNftEventKey, PolygonNftEvents,
        PolygonTransaction, UpdateEdtionTransaction,
    },
    EditionContract, EditionInfo, Services,
};

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
                None => Ok(()),
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
            receiver,
            amount,
            ..
        } = payload;

        let edition_info: proto::EditionInfo =
            edition_info.context("EditionInfo not found in event payload")?;
        let edition_id = Collection::find_max_edition_id(&self.db)
            .await?
            .unwrap_or(0)
            + 1;

        Collection::create(&self.db, collections::Model {
            id: Uuid::from_str(&key.id)?,
            edition_id,
            fee_receiver: fee_receiver.clone(),
            owner: receiver.clone(),
            creator: edition_info.creator.clone(),
            uri: edition_info.uri.clone(),
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
                receiver.parse()?,
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
                    },
                )),
            };

            let key = PolygonNftEventKey {
                id: key.id,
                user_id: key.user_id,
                project_id: key.project_id,
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
            collection: String::new(),
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
            collection_id,
            amount,
        } = payload;

        let collection = Collection::find_by_id(&self.db, collection_id.parse()?)
            .await?
            .context(format!("No collection found for id {collection_id}"))?;

        let typed_tx = self
            .edition_contract
            .mint(
                receiver.parse()?,
                collection.edition_id.into(),
                amount.into(),
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitRetryCreateDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
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
            collection_id,
            receiver,
            amount,
        } = payload;

        let collection = Collection::find_by_id(&self.db, collection_id.parse()?)
            .await?
            .context(format!("No collection found for id {:?}", key.id))?;

        let typed_tx = self
            .edition_contract
            .mint(
                receiver.parse()?,
                collection.edition_id.into(),
                amount.into(),
            )
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitMintDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
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
        let UpdateEdtionTransaction {
            collection_id,
            edition_info,
        } = payload;

        let edition_info = edition_info.context("EditionInfo not found in event payload")?;
        let proto::EditionInfo {
            description,
            image_uri,
            uri,
            creator,
            ..
        } = edition_info.clone();

        let collection = Collection::find_by_id(&self.db, collection_id.parse()?)
            .await?
            .context("collection not found")?;

        let mut collection_am = Collection::get_active_model(collection.clone());
        collection_am.description = Set(description);
        collection_am.image_uri = Set(image_uri);
        collection_am.uri = Set(uri);
        collection_am.creator = Set(creator);
        Collection::update(&self.db, collection_am).await?;

        let typed_tx = self
            .edition_contract
            .edit_edition(collection.edition_id.into(), edition_info.try_into()?)
            .tx;

        if let Some(bytes) = typed_tx.data() {
            let event = PolygonNftEvents {
                event: Some(polygon_nft_events::Event::SubmitUpdateDropTxn(
                    PolygonTransaction {
                        data: bytes.0.to_vec(),
                    },
                )),
            };

            self.producer.send(Some(&event), Some(&key.into())).await?;
        } else {
            bail!("No data in transaction")
        }

        Ok(())
    }
}

impl TryFrom<proto::EditionInfo> for edition_contract::EditionInfo {
    type Error = Error;

    fn try_from(
        proto::EditionInfo {
            description,
            image_uri,
            collection,
            uri,
            creator,
        }: proto::EditionInfo,
    ) -> Result<Self> {
        Ok(Self {
            description,
            image_uri,
            collection,
            uri,
            creator: creator.parse()?,
        })
    }
}

impl From<NftEventKey> for PolygonNftEventKey {
    fn from(
        NftEventKey {
            id,
            user_id,
            project_id,
        }: NftEventKey,
    ) -> Self {
        Self {
            id,
            user_id,
            project_id,
        }
    }
}
