#![deny(clippy::disallowed_methods, clippy::suspicious, clippy::style)]
#![warn(clippy::pedantic, clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

mod collections;
pub mod db;
mod mints;
mod services;
pub use collections::Collection;
use hub_core::prelude::*;
pub use mints::Mint;
mod try_from;
pub use sea_orm;
pub use services::Services;

#[allow(clippy::pedantic)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/polygon_nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/nfts.proto.rs"));
    include!(concat!(env!("OUT_DIR"), "/treasury.proto.rs"));
}

include!(concat!(env!("OUT_DIR"), "/edition_contract.rs"));
