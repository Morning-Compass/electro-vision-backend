use chrono::NaiveDateTime;
use diesel::prelude::QueryableByName;
use serde::Serialize;

#[derive(QueryableByName, Serialize)]
pub struct DbProblem {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub id: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description_multimedia_path: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub worker_username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub mentor_username: String,
}
