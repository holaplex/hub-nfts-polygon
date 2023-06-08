use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Collections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Collections::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Collections::EditionId).integer().not_null())
                    .col(ColumnDef::new(Collections::FeeReceiver).string().not_null())
                    .col(ColumnDef::new(Collections::Owner).string().not_null())
                    .col(ColumnDef::new(Collections::Creator).string().not_null())
                    .col(ColumnDef::new(Collections::Uri).string().not_null())
                    .col(ColumnDef::new(Collections::Description).string().not_null())
                    .col(ColumnDef::new(Collections::ImageUri).string().not_null())
                    .col(
                        ColumnDef::new(Collections::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra("default now()".to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("collections-owner-idx")
                    .table(Collections::Table)
                    .col(Collections::Owner)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("collections-edition_id-idx")
                    .table(Collections::Table)
                    .col(Collections::EditionId)
                    .index_type(IndexType::BTree)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("collections-creator-idx")
                    .table(Collections::Table)
                    .col(Collections::Creator)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Collections::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Collections {
    Table,
    Id,
    EditionId,
    FeeReceiver,
    Owner,
    Creator,
    Uri,
    Description,
    ImageUri,
    CreatedAt,
}
