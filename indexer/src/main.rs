use holaplex_hub_nfts_polygon_core::{db::Connection, proto::PolygonNftEvents};
use holaplex_hub_nfts_polygon_indexer::{process, Args, NftActivityController};
use hub_core::anyhow::Context;
use poem::{listener::TcpListener, middleware::AddData, post, EndpointExt, Route, Server};

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-nfts-polygon",
    };

    hub_core::run(opts, |common, args| {
        let Args {
            db,
            port,
            webhook_signing_key,
            polygon_edition_contract,
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

            let signing_key: Vec<_> = webhook_signing_key.bytes().collect();

            let processor = NftActivityController::new(
                connection,
                producer,
                polygon_edition_contract,
                signing_key,
            );

            let app = Route::new().at(
                "/webhooks/polygon",
                post(process).with(AddData::new(processor)),
            );
            Server::new(TcpListener::bind(format!("0.0.0.0:{port}")))
                .run(app)
                .await
                .map_err(Into::into)
        })
    });
}
