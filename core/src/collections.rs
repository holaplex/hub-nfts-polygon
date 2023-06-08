use holaplex_hub_nfts_polygon_entity::collections::{ActiveModel, Column, Entity, Model};
use sea_orm::{prelude::*, QuerySelect};

use crate::db::Connection;

pub struct Collection;

impl Collection {
    pub async fn create(db: &Connection, model: Model) -> Result<Model, DbErr> {
        let conn = db.get();

        let active_model: ActiveModel = model.into();

        active_model.insert(conn).await
    }

    pub async fn update(db: &Connection, am: ActiveModel) -> Result<Model, DbErr> {
        let conn = db.get();
        am.insert(conn).await
    }

    pub fn get_active_model(model: Model) -> ActiveModel {
        model.into()
    }

    pub async fn find_by_id(db: &Connection, id: Uuid) -> Result<Option<Model>, DbErr> {
        let conn = db.get();

        Entity::find().filter(Column::Id.eq(id)).one(conn).await
    }

    pub async fn find_max_edition_id(db: &Connection) -> Result<Option<i32>, DbErr> {
        let conn = db.get();

        Entity::find()
            .select_only()
            .column_as(Column::EditionId.max(), QueryAs::EditionId)
            .into_values::<_, QueryAs>()
            .one(conn)
            .await
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
enum QueryAs {
    EditionId,
}
