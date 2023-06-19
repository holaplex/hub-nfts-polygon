use std::collections::HashMap;

use hmac::{Hmac, Mac};
use hub_core::{anyhow::Context, serde_json, serde_json::Value};
use poem::Result;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::Signature;

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    Graphql,
    AddressActivity,
    MinedTransaction,
    DroppedTransaction,
    NftMetadataUpdate,
    NftActivity,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub webhook_id: String,
    pub id: String,
    pub created_at: String,
    #[serde(rename = "type")]
    pub ty: EventType,
    pub event: EventPayload,
}

impl Payload {
    pub fn verify(&self, signature: &Signature, signing_key: &[u8]) -> Result<()> {
        let bytes = serde_json::to_vec(&self).context("failed to serialize payload")?;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(signing_key).context("failed to build hmac")?;
        mac.update(&bytes);
        mac.verify_slice(&signature.0)
            .context("Invalid message received")?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventPayload {
    pub network: String,
    pub activity: Vec<ActivityPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPayload {
    pub from_address: String,
    pub to_address: String,
    pub contract_address: String,
    pub hash: String,
    pub category: TokenStandard,
    pub erc1155_metadata: Option<Vec<ERC1155Metadata>>,
    pub erc721_token_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ERC1155Metadata {
    pub token_id: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TokenStandard {
    Erc1155,
    Erc721,
}
