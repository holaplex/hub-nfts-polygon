mod error;
mod handler;
mod types;

use error::Error;
pub use handler::{process, NftActivityController};
use hmac::{Hmac, Mac};
use holaplex_hub_nfts_polygon_core::db::DbArgs;
use hub_core::{anyhow::Context, clap};
use poem::{FromRequest, Request, RequestBody, Result};
use sha2::Sha256;

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[command(flatten)]
    pub db: DbArgs,

    #[arg(short, long, env, default_value_t = 4000)]
    pub port: u16,

    #[arg(long, env)]
    pub webhook_signing_key: String,

    #[arg(long, env)]
    pub polygon_edition_contract: String,
}

pub struct PayloadBytes(Vec<u8>);

impl PayloadBytes {
    pub fn verify(&self, signature: &Signature, signing_key: &[u8]) -> Result<()> {
        let bytes = &self.0;
        let mut mac =
            Hmac::<Sha256>::new_from_slice(signing_key).context("failed to build hmac")?;
        mac.update(bytes);
        mac.verify_slice(&signature.0)
            .context("Invalid message received")?;

        Ok(())
    }
}

#[poem::async_trait]
impl<'a> FromRequest<'a> for PayloadBytes {
    async fn from_request(_req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let payload = body.take()?.into_bytes().await?;

        Ok(PayloadBytes(payload.to_vec()))
    }
}

#[derive(Debug, Clone)]
pub struct Signature(Vec<u8>);

#[poem::async_trait]
impl<'a> FromRequest<'a> for Signature {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let signature = req
            .headers()
            .get("X-Alchemy-Signature")
            .ok_or(Error::MissingHeader)?
            .to_str()
            .map_err(|_| Error::InvalidUtf8)?;

        let bytes = hex::decode(signature).map_err(|_| Error::InvalidHexadecimal)?;

        Ok(Signature(bytes))
    }
}
