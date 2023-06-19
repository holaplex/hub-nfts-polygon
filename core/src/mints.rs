use holaplex_hub_nfts_polygon_entity::{
    collections::{self, Model as Collection},
    mints::{ActiveModel, Column, Entity, Model, Relation},
    prelude::Collections,
};
use sea_orm::{
    prelude::*, ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect,
    RelationTrait,
};

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

    pub async fn get_mints_for_edition(
        db: &Connection,
        owner: &str,
        edition_id: u64,
        value: u64,
    ) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .join(JoinType::InnerJoin, Relation::Collection.def())
            .filter(collections::Column::EditionId.eq(edition_id))
            .filter(Column::Owner.eq(owner))
            .limit(Some(value))
            .all(db.get())
            .await
    }
}
