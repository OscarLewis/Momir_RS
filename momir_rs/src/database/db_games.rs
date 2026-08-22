use sea_orm::entity::prelude::*;

#[allow(dead_code)]
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub start_date: DateTimeUtc,

    #[sea_orm(has_many)]
    pub cards: HasMany<super::db_cards::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
