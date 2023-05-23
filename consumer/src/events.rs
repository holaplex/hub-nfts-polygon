
use hub_core::{
    prelude::*,
    uuid::Uuid,
};

use crate::{db::Connection, Services};

/// Process the given message for various services.
///
/// # Errors
/// This functioncan return an error if it fails to process any event
pub async fn process(msg: Services, db: Connection) -> Result<()> {
    // match topics
    Ok(())
}
