use crate::models::{self, AuthUser as User, FullUser, Workspace};
use crate::{est_conn, schema, DPool};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel::{result::Error as DieselError, ExpressionMethods};
use schema::auth_users as users_data;
use schema::auth_users::dsl as users_table;
use schema::workspaces as workspaces_data;
use schema::workspaces::dsl as workspaces_table;

pub struct FindData {}

pub trait Find {
    async fn exists_by_email(email: String, pool: DPool) -> Result<bool, DieselError>;
    async fn find_auth_user_by_email(email: String, pool: DPool) -> Result<User, DieselError>;
    async fn find_workspace_by_owner_email(
        email: String,
        pool: DPool,
    ) -> Result<Workspace, DieselError>;
    async fn find_full_user_by_email(email: String, pool: DPool) -> Result<FullUser, DieselError>;
    async fn find_auth_user_by_id(id: i32, pool: DPool) -> Result<User, DieselError>;
}

impl Find for FindData {
    async fn find_auth_user_by_id(uid: i32, pool: DPool) -> Result<User, DieselError> {
        let conn = &mut est_conn(pool.clone());
        let user_data = users_table::auth_users
            .filter(users_data::id.eq(uid))
            .select(User::as_select())
            .first(conn);
        match user_data {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }
    async fn find_full_user_by_email(email: String, pool: DPool) -> Result<FullUser, DieselError> {
        todo!()
    }
    async fn find_workspace_by_owner_email(
        _email: String,
        pool: DPool,
    ) -> Result<Workspace, DieselError> {
        let user = match self::FindData::find_auth_user_by_email(_email, pool.clone()).await {
            Ok(usr) => usr,
            Err(e) => return Err(e),
        };
        let conn = &mut est_conn(pool.clone());
        let workspace: models::Workspace = match workspaces_table::workspaces
            .filter(workspaces_data::owner_id.eq(user.id))
            .select(models::Workspace::as_select())
            .first(conn)
        {
            Ok(workspace) => workspace,
            Err(e) => return Err(e),
        };

        Ok(workspace)
    }

    async fn exists_by_email(_email: String, pool: DPool) -> Result<bool, DieselError> {
        use schema::auth_users::dsl::*;
        let conn = &mut est_conn(pool.clone());
        let is_found = diesel::select(diesel::dsl::exists(
            auth_users.select(email).filter(email.eq(_email.clone())),
        ))
        .get_result::<bool>(conn);
        match is_found {
            Ok(true) => {
                println!("User found.");
                Ok(true)
            }
            Ok(false) => {
                println!("User not found.");
                Ok(false)
            }
            Err(e) => Err(e),
        }
    }
    async fn find_auth_user_by_email(_email: String, pool: DPool) -> Result<User, DieselError> {
        let conn = &mut est_conn(pool.clone());
        let user_data = users_table::auth_users
            .filter(users_data::email.eq(_email))
            .select(User::as_select())
            .first(conn);
        match user_data {
            Ok(user) => Ok(user),
            Err(e) => Err(e),
        }
    }
}
