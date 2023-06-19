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

            let app = Route::new().at(
                "/",
                post(process)
                    .with(AddData::new(connection))
                    .with(AddData::new(producer)),
            );
            Server::new(TcpListener::bind(indexer_server_address))
                .run(app)
                .await
                .map_err(Into::into)
        })
    });
}
