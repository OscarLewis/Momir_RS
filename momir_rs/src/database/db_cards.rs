use sea_orm::entity::prelude::*;

#[allow(dead_code)]
#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cards")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub game_id: i32,

    #[sea_orm(default_value = 0)]
    pub count: i32,

    pub name: String,

    #[sea_orm(belongs_to, from = "game_id", to = "id")]
    pub game: BelongsTo<super::db_games::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
