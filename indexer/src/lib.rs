mod handler;
mod types;

use std::net::SocketAddr;

pub use handler::process;
use holaplex_hub_nfts_polygon_core::db::DbArgs;
use hub_core::clap;
use poem::{FromRequest, Request, RequestBody, Result};
use types::Payload;

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[command(flatten)]
    pub db: DbArgs,

    #[arg(long, default_value = "0.0.0.0:4000", env)]
    pub indexer_server_address: SocketAddr,
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for Payload {
    async fn from_request(_req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let payload = body.take()?.into_json().await?;

        Ok(payload)
    }
}
