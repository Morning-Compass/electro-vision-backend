use chrono::NaiveDateTime;
use diesel::prelude::QueryableByName;
use serde::Serialize;

#[derive(QueryableByName, Serialize)]
pub struct DbTask {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub title: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description_multimedia_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
    pub due_date: Option<NaiveDateTime>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub importance: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub assigner_username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub assignee_username: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub category: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub created_at: NaiveDateTime,
}
