use chrono::Offset;
use sea_orm::{ConnectionTrait, QueryFilter, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, Set, Unchanged};
use entity::{chart_data, charts};
use migration::sea_query;

pub async fn insert_data_many<C, D>(db: &C, data: D) -> Result<(), DbErr>
where
    C: ConnectionTrait,
    D: IntoIterator<Item = chart_data::ActiveModel> + Send + Sync,
{
    let mut data = data.into_iter().peekable();
    if data.peek().is_some() {
        chart_data::Entity::insert_many(data)
            .on_conflict(
                sea_query::OnConflict::columns([
                    chart_data::Column::ChartId,
                    chart_data::Column::Date,
                ])
                    .update_column(chart_data::Column::Value)
                    .update_column(chart_data::Column::MinBlockscoutBlock)
                    .to_owned(),
            )
            .exec(db)
            .await?;
    }
    Ok(())
}

pub async fn set_last_updated_at<Tz>(
    chart_id: i32,
    db: &DatabaseConnection,
    at: chrono::DateTime<Tz>,
) -> Result<(), DbErr>
where
    Tz: chrono::TimeZone,
{
    let last_updated_at = at.with_timezone(&chrono::Utc.fix());
    let model = charts::ActiveModel {
        id: Unchanged(chart_id),
        last_updated_at: Set(Some(last_updated_at)),
        ..Default::default()
    };
    charts::Entity::update(model)
        .filter(charts::Column::Id.eq(chart_id))
        .exec(db)
        .await?;
    Ok(())
}
