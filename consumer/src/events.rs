use crate::{
    db::Connection,
    edition_contract,
    proto::{nft_events::Event as NftEvents, CreateEditionTransaction, EditionInfo, NftEventKey},
    Services,
};
use ethers::{
    middleware::SignerMiddleware,
    prelude::k256::ecdsa::SigningKey,
    providers::{Http, Middleware, Provider},
    signers::{Signer, Wallet},
    types::{Address, U256},
};
use hub_core::{anyhow::Error, prelude::*, serde_json};

/// Process the given message for various services.
///
/// # Errors
/// This function can return an error if it fails to process any event
pub async fn process(
    msg: Services,
    edition_contract: Arc<edition_contract::EditionContract<Provider<Http>>>,
    // _db: Connection,
) -> Result<()> {
    // match topics
    match msg {
        Services::Nfts(key, e) => match e.event {
            Some(NftEvents::CreatePolygonEdition(payload)) => {
                handle_create_polygon_edition(edition_contract, key, payload).await
            },
            Some(_) | None => Ok(()),
        },
    }
}

async fn handle_create_polygon_edition(
    edition_contract: Arc<edition_contract::EditionContract<Provider<Http>>>,
    _key: NftEventKey,
    payload: CreateEditionTransaction,
) -> Result<()> {
    let CreateEditionTransaction {
        edition_id,
        edition_info,
        fee_receiver,
        fee_numerator,
        receiver,
        amount,
        ..
    } = payload;

    let edition_info = edition_info
        .ok_or_else(|| anyhow!("no edition info"))?
        .try_into()?;

    let id = U256::from(edition_id);
    let token_receiver: Address = receiver.parse()?;
    let to_mint_amount = U256::from(amount);
    let fee_receiver: Address = fee_receiver.parse()?;
    let fee_numerator: u128 = fee_numerator.try_into()?;

    let tx = edition_contract
        .create_edition(
            id,
            edition_info,
            token_receiver,
            to_mint_amount,
            fee_receiver,
            fee_numerator,
        )
        .tx;

    // TODO: in order to test set this to the private key used to deploy the contract on mumbai
    // This is example code to test if transactions are being sent correctly
    let wallet: Wallet<SigningKey> =
        "".parse()?;
    wallet.with_chain_id(80001 as u64);
    let signed_transaction = wallet.sign_transaction(&tx).await?;

    let client = SignerMiddleware::new(edition_contract.client(), wallet.with_chain_id(80001));

    let pending_tx = client.send_transaction(tx, None).await?;

    let receipt = pending_tx
        .await?
        .ok_or_else(|| anyhow!("tx dropped from mempool"))?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    info!("Sent tx: {}\n", serde_json::to_string(&tx)?);
    info!("Tx receipt: {}", serde_json::to_string(&receipt)?);

    Ok(())
}

impl TryFrom<EditionInfo> for edition_contract::EditionInfo {
    type Error = Error;

    fn try_from(
        EditionInfo {
            description,
            image_uri,
            collection,
            uri,
            creator,
        }: EditionInfo,
    ) -> Result<Self> {
        Ok(Self {
            description,
            image_uri,
            collection,
            uri,
            creator: creator.parse()?,
        })
    }
}
