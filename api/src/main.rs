

use holaplex_rust_boilerplate_api::{
    db::{Connection, DbArgs},
    graphql::schema::build_schema,
    handlers::{graphql_handler, playground},
    AppState,
};
use hub_core::{clap, prelude::*};
use poem::{
    get, listener::TcpListener, middleware::AddData, post, EndpointExt, Route, Server,
};

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[arg(short, long, env, default_value_t = 3002)]
    pub port: u16,

    #[command(flatten)]
    pub db: DbArgs,
}

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-boilerplate-rust",
    };

    hub_core::run(opts, |common, args| {
        let Args { port, db } = args;

        common.rt.block_on(async move {
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;

            let schema = build_schema();

            let state = AppState::new(schema, connection);

            Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
                .run(
                    Route::new()
                        .at(
                            "/graphql",
                            post(graphql_handler).with(AddData::new(state.clone())),
                        )
                        .at("/playground", get(playground)),
                )
                .await
                .map_err(Into::into)
        })
    });
}
