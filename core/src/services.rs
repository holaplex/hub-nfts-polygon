use hub_core::{consumer::RecvError, tracing::info};
use prost::Message;

use crate::proto::{self, PolygonNftEventKey, PolygonNftEvents};

#[derive(Debug)]
pub enum Services {
    Nfts(proto::NftEventKey, proto::PolygonEvents),
    Treasuries(proto::TreasuryEventKey, proto::TreasuryEvents),
}

impl hub_core::consumer::MessageGroup for Services {
    const REQUESTED_TOPICS: &'static [&'static str] = &["hub-nfts", "hub-treasuries"];

    fn from_message<M: hub_core::consumer::Message>(msg: &M) -> Result<Self, RecvError> {
        let topic = msg.topic();
        let key = msg.key().ok_or(RecvError::MissingKey)?;
        let val = msg.payload().ok_or(RecvError::MissingPayload)?;
        info!(topic, ?key, ?val);

        match topic {
            "hub-nfts" => {
                let key = proto::NftEventKey::decode(key)?;
                let val = proto::PolygonEvents::decode(val)?;

                Ok(Services::Nfts(key, val))
            },
            "hub-treasuries" => {
                let key = proto::TreasuryEventKey::decode(key)?;
                let val = proto::TreasuryEvents::decode(val)?;

                Ok(Services::Treasuries(key, val))
            },
            t => Err(RecvError::BadTopic(t.into())),
        }
    }
}

impl hub_core::producer::Message for PolygonNftEvents {
    type Key = PolygonNftEventKey;
}
