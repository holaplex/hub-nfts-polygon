use holaplex_hub_nfts_polygon_entity::{
    collections::Model as Collection,
    mints::{ActiveModel, Column, Entity, Model},
    prelude::Collections,
};
use sea_orm::prelude::*;

use crate::db::Connection;

pub struct Mint;

impl Mint {
    pub async fn create(db: &Connection, model: Model) -> Result<Model, DbErr> {
        let conn = db.get();

        let active_model: ActiveModel = model.into();

        active_model.insert(conn).await
    }

    pub async fn find_by_id(db: &Connection, id: Uuid) -> Result<Option<Model>, DbErr> {
        let conn = db.get();

        Entity::find().filter(Column::Id.eq(id)).one(conn).await
    }

    pub async fn find_with_collection(
        db: &Connection,
        id: Uuid,
    ) -> Result<Option<(Model, Option<Collection>)>, DbErr> {
        let conn: &DatabaseConnection = db.get();

        Entity::find()
            .filter(Column::Id.eq(id))
            .find_also_related(Collections)
            .one(conn)
            .await
    }
}
