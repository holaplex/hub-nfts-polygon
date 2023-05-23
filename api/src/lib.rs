#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod graphql;
pub mod db;
pub mod handlers;


use db::Connection;
use hub_core::{
    anyhow::{Error, Result},
    clap,
    prelude::*,
    uuid::Uuid,
};
use poem::{async_trait, FromRequest, Request, RequestBody};

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[arg(short, long, env, default_value_t = 3002)]
    pub port: u16,

    #[command(flatten)]
    pub db: db::DbArgs,
}

#[derive(Debug, Clone, Copy)]
pub struct UserID(Option<Uuid>);

impl TryFrom<&str> for UserID {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let id = Uuid::from_str(value)?;

        Ok(Self(Some(id)))
    }
}

#[async_trait]
impl<'a> FromRequest<'a> for UserID {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let id = req
            .headers()
            .get("X-USER-ID")
            .and_then(|value| value.to_str().ok())
            .map_or(Ok(Self(None)), Self::try_from)?;

        Ok(id)
    }
}

#[derive(Debug, Clone)]
pub struct UserEmail(Option<String>);

#[async_trait]
impl<'a> FromRequest<'a> for UserEmail {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let id = req
            .headers()
            .get("X-USER-EMAIL")
            .and_then(|value| value.to_str().ok())
            .map(std::string::ToString::to_string);

        Ok(Self(id))
    }
}

#[derive(Clone)]
pub struct AppState {
    pub schema: graphql::schema::AppSchema,
    pub connection: Connection,
}

impl AppState {
    #[must_use]
    pub fn new(
        schema: graphql::schema::AppSchema,
        connection: Connection,
    ) -> Self {
        Self {
            schema,
            connection,
        }
    }
}

pub struct AppContext {
    pub db: Connection,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
}

impl AppContext {
    #[must_use] pub fn new(db: Connection, user_id: Option<Uuid>, user_email: Option<String>) -> Self {
        Self {
            db,
            user_id,
            user_email,
        }
    }
}
