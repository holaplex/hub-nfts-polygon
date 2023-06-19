use holaplex_hub_nfts_polygon_core::{db::Connection, proto::PolygonNftEvents};
use holaplex_hub_nfts_polygon_indexer::{process, Args};
use hub_core::anyhow::Context;
use poem::{listener::TcpListener, middleware::AddData, post, EndpointExt, Route, Server};

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-nfts-polygon",
    };

    hub_core::run(opts, |common, args| {
        let Args {
            db,
            indexer_server_address,
            alchemy_signing_key,
        } = args;

        common.rt.block_on(async move {
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;

            let producer = common
                .producer_cfg
                .clone()
                .build::<PolygonNftEvents>()
                .await?;

            let signing_key: Vec<_> = alchemy_signing_key.bytes().collect();

            let app = Route::new().at(
                "/",
                post(process)
                    .with(AddData::new(connection))
                    .with(AddData::new(producer))
                    .with(AddData::new(signing_key)),
            );
            Server::new(TcpListener::bind(indexer_server_address))
                .run(app)
                .await
                .map_err(Into::into)
        })
    });
}
