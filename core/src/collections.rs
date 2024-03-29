use holaplex_hub_nfts_polygon_entity::{
    collections::{ActiveModel, Column, Entity, Model, Relation},
    mints,
};
use sea_orm::{prelude::*, JoinType, QuerySelect};

use crate::db::Connection;

pub struct Collection;

impl Collection {
    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn create(db: &Connection, model: Model) -> Result<Model, DbErr> {
        let conn = db.get();

        let active_model: ActiveModel = model.into();

        active_model.insert(conn).await
    }

    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn update(db: &Connection, am: ActiveModel) -> Result<Model, DbErr> {
        let conn = db.get();
        am.update(conn).await
    }

    #[must_use]
    pub fn get_active_model(model: Model) -> ActiveModel {
        model.into()
    }

    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn find_by_id(db: &Connection, id: Uuid) -> Result<Option<Model>, DbErr> {
        let conn = db.get();

        Entity::find().filter(Column::Id.eq(id)).one(conn).await
    }

    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn find_max_edition_id(db: &Connection) -> Result<Option<i32>, DbErr> {
        let conn = db.get();

        let v: Option<Option<i32>> = Entity::find()
            .select_only()
            .column_as(Column::EditionId.max(), QueryAs::EditionId)
            .into_values::<_, QueryAs>()
            .one(conn)
            .await?;

        Ok(v.flatten())
    }

    /// Res
    ///
    /// # Errors
    /// This function fails if ...
    pub async fn find_by_mint_id(db: &Connection, mint_id: Uuid) -> Result<Option<Model>, DbErr> {
        let conn = db.get();

        Entity::find()
            .join(JoinType::InnerJoin, Relation::Mints.def())
            .filter(mints::Column::Id.eq(mint_id))
            .one(conn)
            .await
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    EditionId,
}
