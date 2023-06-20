#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

pub mod events;

use ethers::providers::{Http, Provider};
use holaplex_hub_nfts_polygon_core::edition_contract;
use hub_core::prelude::*;

pub type EditionContract = Arc<edition_contract::EditionContract<Provider<Http>>>;
