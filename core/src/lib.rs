#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

mod collections;
pub mod db;
mod mints;
mod services;
pub use collections::Collection;
use hub_core::prelude::*;
pub use mints::Mint;
pub use sea_orm;
pub use services::Services;

use crate::proto::{NftEventKey, PolygonNftEventKey, TreasuryEventKey};

#[allow(clippy::pedantic)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/polygon_nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/treasury.proto.rs"));
}

include!(concat!(env!("OUT_DIR"), "/edition_contract.rs"));

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
impl From<TreasuryEventKey> for PolygonNftEventKey {
    fn from(
        TreasuryEventKey {
            id,
            user_id,
            project_id,
        }: TreasuryEventKey,
    ) -> Self {
        Self {
            id,
            user_id,
            project_id,
        }
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
