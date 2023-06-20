use std::sync::Arc;

use ethers::{providers::Provider, types::Address};
use holaplex_hub_nfts_polygon::events::Processor;
use holaplex_hub_nfts_polygon_core::{
    db::{Connection, DbArgs},
    edition_contract,
    proto::PolygonNftEvents,
    Services,
};
use hub_core::{clap, prelude::*, tokio};

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[command(flatten)]
    pub db: DbArgs,

    #[arg(long, env)]
    pub polygon_edition_contract: String,

    #[arg(long, env)]
    pub polygon_rpc_endpoint: String,
}

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-nfts-polygon",
    };

    hub_core::run(opts, |common, args| {
        let Args {
            db,
            polygon_edition_contract,
            polygon_rpc_endpoint,
        } = args;

        common.rt.block_on(async move {
            let edition_contract_address: Address = polygon_edition_contract.parse()?;

            let provider = Arc::new(Provider::try_from(polygon_rpc_endpoint)?);
            let edition_contract = Arc::new(edition_contract::EditionContract::new(
                edition_contract_address,
                provider,
            ));
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;

            let cons = common.consumer_cfg.build::<Services>().await?;
            let producer = common
                .producer_cfg
                .clone()
                .build::<PolygonNftEvents>()
                .await?;
            let event_processor = Processor::new(connection, producer, edition_contract);

            let mut stream = cons.stream();
            loop {
                let event_processor = event_processor.clone();

                match stream.next().await {
                    Some(Ok(msg)) => {
                        info!(?msg, "message received");

                        tokio::spawn(async move { event_processor.process(msg).await });
                        tokio::task::yield_now().await;
                    },
                    None => (),
                    Some(Err(e)) => {
                        warn!("failed to get message {:?}", e);
                    },
                }
            }
        })
    });
}
