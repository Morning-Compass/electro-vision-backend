use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workspace_users)]
pub struct WorkspaceUser {
    pub user_id: i32,
    pub workspace_id: i32,
    pub plane_file_cut_name: Option<String>,
    pub workspace_role_id: i32,
    pub position_id: Option<i32>,
    pub checkin_time: Option<NaiveTime>,
    pub checkout_time: Option<NaiveTime>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::workspace_roles)]
pub struct WorkspaceRole {
    pub user_id: i32,
    pub name: String,
}
