// add workspace struct
//

use chrono::{NaiveDate, NaiveDateTime};

use crate::{models::AuthUser, DPool};

pub enum WorkspaceError {
    ServerError,
}

pub struct WorkspaceData {
    pub id: i32,
    pub plan_file_name: String,
    pub start_date: NaiveDateTime,
    pub finish_date: Option<NaiveDate>,
    pub geolocation: Option<String>,
    pub owner_id: i32,
    pub ev_subscription_id: i32,
    pub name: String,
}

pub trait Workspace {
    async fn list_users(pool: DPool) -> Result<Vec<AuthUser>, WorkspaceError>;
}

impl Workspace for WorkspaceData {
    async fn list_users(pool: DPool) -> Result<Vec<AuthUser>, WorkspaceError> {
        todo!()
    }
}
