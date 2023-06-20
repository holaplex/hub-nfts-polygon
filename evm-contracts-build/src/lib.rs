use std::path::PathBuf;

use ethers::prelude::Abigen;
use eyre::{Error, Result};

const EDITION_CONTRACT_OUT_FILE: &str = "edition_contract.rs";

pub fn run(edition_contract_url: &str) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let abi_source = reqwest::get(edition_contract_url).await?.text().await?;
            let out_dir = PathBuf::try_from(std::env::var("OUT_DIR")?)?;
            let out_file = out_dir.join(EDITION_CONTRACT_OUT_FILE);

            if out_file.exists() {
                std::fs::remove_file(&out_file)?;
            }

            Abigen::new("EditionContract", abi_source)?
                .generate()?
                .write_to_file(out_file)?;

            Ok::<(), Error>(())
        })?;

    Ok(())
}
