use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::Insertable;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workspaces)]
pub struct NewWorkspace<'a> {
    pub owner_id: i32,
    pub geolocation: Option<&'a str>,
    pub plan_file_name: Option<&'a str>,
    pub start_date: NaiveDateTime,
    pub finish_date: Option<NaiveDate>,
    pub ev_subscription_id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workspace_invitations)]
pub struct NewInvitation {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub workspace_id: i32,
}
