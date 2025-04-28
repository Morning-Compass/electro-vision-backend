use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable, Clone)]
#[diesel(table_name = crate::schema::auth_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuthUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Role {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::user_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::confirmation_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConfirmationToken {
    pub id: i32,
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub confirmed_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::workspace_invitations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceInvitation {
    pub id: i32,
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub confirmed_at: Option<NaiveDateTime>,
    pub workspace_id: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::password_reset_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PasswordResetTokens {
    pub id: i32,
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub confirmed_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::workspaces)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Workspace {
    pub id: i32,
    pub plan_file_name: String,
    pub start_date: NaiveDateTime,
    pub finish_date: Option<NaiveDate>,
    pub geolocation: Option<String>,
    pub owner_id: i32,
    pub ev_subscription_id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: i32,
    pub workspace_id: i32,
    pub assigner_id: i32,
    pub worker_id: i32,
    pub description: Option<String>,
    pub description_multimedia: Option<Vec<u8>>,
    pub assignment_date: NaiveDateTime,
    pub due_date: Option<NaiveDateTime>,
    pub status_id: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Status {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::importance)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Importance {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::workspace_roles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceRole {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::ev_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EvSubscription {
    pub id: i32,
    pub subscription: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::full_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FullUser {
    pub user_id: i32,
    pub phone: String,
    pub phonde_dial_code_id: i32,
    pub countru_of_origin_id: i32,
    pub title: Option<String>,
    pub education: Option<String>,
    pub birth_date: NaiveDateTime,
    pub account_bank_number: Option<String>,
    pub photo: Option<Vec<u8>>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::countries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::phone_dial_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PhoneDialCode {
    pub id: i32,
    pub code: String,
    pub country: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::problems)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Problem {
    pub id: i32,
    pub worker_id: i32,
    pub description: Option<String>,
    pub mentor_id: i32,
    pub problem_multimedia: Option<Vec<u8>>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::worker_workspace_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkerWorkspaceData {
    pub employer_id: i32,
    pub user_id: i32,
    pub working_since: NaiveDateTime,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::positions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Position {
    pub id: i32,
    pub workspace_id: i32,
    pub name: Option<String>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::workspace_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceUser {
    pub user_id: i32,
    pub workspace_id: i32,
    pub plane_file_cut_name: Option<String>,
    pub workspace_role_id: i32,
    pub position_id: Option<i32>,
    pub checkin_time: Option<NaiveTime>,
    pub checkout_time: Option<NaiveTime>,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::conversations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Conversation {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: i32,
    pub conversation_id: i32,
    pub sender_id: i32,
    pub body: String,
    pub read: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::conversation_participants)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ConversationParticipant {
    pub user_id: i32,
    pub conversation_id: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users_citizenships)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserCitizenship {
    pub country_id: i32,
    pub user_id: i32,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::tasks_category)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskCategory {
    pub id: i32,
    pub workspace_id: i32,
    pub name: String,
}

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::attendance)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Attendance {
    pub id: i32,
    pub user_id: i32,
    pub date: NaiveDate,
    pub checkin: NaiveTime,
    pub checkin_photo: Option<Vec<u8>>,
    pub checkout: Option<NaiveTime>,
    pub checkout_photo: Option<Vec<u8>>,
    pub workspace_id: i32,
}
