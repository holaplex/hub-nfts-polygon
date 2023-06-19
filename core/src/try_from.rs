use hub_core::prelude::*;

use crate::{
    edition_contract,
    proto::{self, NftEventKey, PolygonNftEventKey, TreasuryEventKey},
};
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
