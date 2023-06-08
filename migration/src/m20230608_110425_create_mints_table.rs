use sea_orm_migration::prelude::*;

use crate::m20230608_110420_create_collections_table::Collections;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mints::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Mints::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Mints::CollectionId).uuid().not_null())
                    .col(ColumnDef::new(Mints::Owner).text().not_null())
                    .col(ColumnDef::new(Mints::Amount).integer().not_null())
                    .col(
                        ColumnDef::new(Mints::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra("default now()".to_string()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mints_collection_id")
                            .from(Mints::Table, Mints::CollectionId)
                            .to(Collections::Table, Collections::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("mints_collection_id_idx")
                    .table(Mints::Table)
                    .col(Mints::CollectionId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("mints_owner_idx")
                    .table(Mints::Table)
                    .col(Mints::Owner)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mints::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Mints {
    Table,
    Id,
    CollectionId,
    Owner,
    Amount,
    CreatedAt,
}
