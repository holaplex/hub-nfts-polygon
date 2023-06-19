use std::collections::HashMap;

use hub_core::serde_json::Value;
use serde::{Deserialize, Serialize};

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
