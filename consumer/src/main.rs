use holaplex_hub_nfts_polygon::{
    db::{Connection, DbArgs},
    events, Services,
};
use hub_core::{clap, prelude::*, tokio};

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
            let connection = Connection::new(db)
                .await
                .context("failed to get database connection")?;

            let cons = common.consumer_cfg.build::<Services>().await?;

            let mut stream = cons.stream();
            loop {
                let connection = connection.clone();

                match stream.next().await {
                    Some(Ok(msg)) => {
                        info!(?msg, "message received");

                        tokio::spawn(async move { events::process(msg, connection.clone()).await });
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
