use ethers::{
    providers::{Http, Provider},
    types::Address,
};
use holaplex_hub_nfts_polygon::{
    db::{Connection, DbArgs},
    edition_contract, events, Services,
};
use hub_core::{clap, prelude::*, tokio};
use std::sync::Arc;

#[derive(Debug, clap::Args)]
#[command(version, author, about)]
pub struct Args {
    #[command(flatten)]
    pub db: DbArgs,
}

pub fn main() {
    let opts = hub_core::StartConfig {
        service_name: "hub-nfts-polygon",
    };

    hub_core::run(opts, |common, args| {
        let Args { db } = args;

        common.rt.block_on(async move {
            // let connection = Connection::new(db)
            //     .await
            //     .context("failed to get database connection")?;
            // TODO: move to env configuration but is the address of the proxy contract for the editions contract
            let edition_contract_address: Address =
                "0xeF3EB73e28afa4be176Be38B0690868Da6b818F4".parse()?;
            let rpc_url = "https://rpc-mumbai.maticvigil.com";
            let provider = Arc::new(Provider::try_from(rpc_url)?);
            let edition_contract = Arc::new(edition_contract::EditionContract::new(
                edition_contract_address,
                provider,
            ));

            let cons = common.consumer_cfg.build::<Services>().await?;

            let mut stream = cons.stream();
            loop {
                // let connection = connection.clone();
                let edition_contract = edition_contract.clone();

                match stream.next().await {
                    Some(Ok(msg)) => {
                        info!(?msg, "message received");

                        tokio::spawn(async move { events::process(msg, edition_contract).await });
                        tokio::task::yield_now().await;
                    },
                    None => (),
                    Some(Err(e)) => {
                        warn!("failed to get message {:?}", e);
                    },
                }
            }

            Ok(())
        })
    });
}
