#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod db;
pub mod events;

use hub_core::{
    clap,
    prelude::*,
    consumer::RecvError,
};

#[allow(clippy::pedantic)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/nfts.proto.rs"));
}


#[derive(Debug)]
pub enum Services {
    Nfts(proto::NftEventKey, proto::NftEvents),
}

impl hub_core::consumer::MessageGroup for Services {
    const REQUESTED_TOPICS: &'static [&'static str] = &["hub-nfts"];

    fn from_message<M: hub_core::consumer::Message>(msg: &M) -> Result<Self, RecvError> {
        let topic = msg.topic();
        let key = msg.key().ok_or(RecvError::MissingKey)?;
        let val = msg.payload().ok_or(RecvError::MissingPayload)?;
        info!(topic, ?key, ?val);

        match topic {
            "hub-nfts" => {
                let key = proto::NftEventKey::decode(key)?;
                let val = proto::NftEvents::decode(val)?;

                Ok(Services::Nfts(key, val))
            },
            t => Err(RecvError::BadTopic(t.into())),
        }
    }
}


#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {

    #[command(flatten)]
    pub db: db::DbArgs,
}
