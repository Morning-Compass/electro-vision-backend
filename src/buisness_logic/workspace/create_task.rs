use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::models_insertable;
use crate::response::Response as Res;
use crate::schema::tasks::dsl as tasks_table;
use crate::schema::tasks_category as tasks_category_data;
use crate::schema::tasks_category::dsl as tasks_category_table;
use crate::schema::workspaces as workspaces_data;
use crate::schema::workspaces::dsl as workspaces_table;

use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use chrono::Utc;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error as DieselError;
use diesel::QueryDsl;
use diesel::{Connection, ExpressionMethods, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Status {
    HelpNeeded,
    Todo,
    InProgress,
    Completed,
    Canceled,
}

#[derive(Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Importance {
    Low,
    Medium,
    High,
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    workspace_id: i32,
    assigner_email: String,
    assignee_email: String,
    description: Option<String>,
    description_multimedia: Option<Vec<u8>>,
    due_date: Option<NaiveDateTime>,
    status: Option<Status>,
    title: String,
    importance: Option<Importance>,
    category: Option<String>,
}

#[post("/workspace/create/task")]
pub async fn create_task(pool: DPool, req: Json<CreateTaskRequest>) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());

    let assigner =
        match <FindData as Find>::find_auth_user_by_email(req.assigner_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Assigner not found")),
        };

    let assignee =
        match <FindData as Find>::find_auth_user_by_email(req.assignee_email.clone(), pool.clone())
            .await
        {
            Ok(user) => user,
            Err(_) => return HttpResponse::BadRequest().json(Res::new("Assignee not found")),
        };

    let result = conn.transaction::<_, DieselError, _>(|conn| {
        let workspace_exists: bool = diesel::select(diesel::dsl::exists(
            workspaces_table::workspaces.filter(workspaces_data::id.eq(req.workspace_id)),
        ))
        .get_result(conn)?;

        if !workspace_exists {
            return Err(DieselError::NotFound);
        }

        let category_id = match req.category.as_deref() {
            Some(category_name) => {
                match tasks_category_table::tasks_category
                    .filter(tasks_category_data::workspace_id.eq(req.workspace_id))
                    .filter(tasks_category_data::name.eq(category_name))
                    .select(tasks_category_data::id)
                    .first::<i32>(conn)
                {
                    Ok(id) => id,
                    Err(_) => diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(req.workspace_id),
                            tasks_category_data::name.eq(category_name),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                }
            }
            None => {
                // Try to find or create "NONE" category
                match tasks_category_table::tasks_category
                    .filter(tasks_category_data::workspace_id.eq(req.workspace_id))
                    .filter(tasks_category_data::name.eq("NONE"))
                    .select(tasks_category_data::id)
                    .first::<i32>(conn)
                {
                    Ok(id) => id,
                    Err(_) => diesel::insert_into(tasks_category_table::tasks_category)
                        .values((
                            tasks_category_data::workspace_id.eq(req.workspace_id),
                            tasks_category_data::name.eq("NONE"),
                        ))
                        .returning(tasks_category_data::id)
                        .get_result(conn)?,
                }
            }
        };

        let status_id = req.status.as_ref().map_or(2, |s| match s {
            Status::HelpNeeded => 1,
            Status::Todo => 2,
            Status::InProgress => 3,
            Status::Completed => 4,
            Status::Canceled => 5,
        });

        let importance_id = req.importance.as_ref().map_or(2, |i| match i {
            Importance::Low => 1,
            Importance::Medium => 2,
            Importance::High => 3,
        });

        let new_task = models_insertable::Task {
            workspace_id: req.workspace_id,
            assigner_id: assigner.id,
            worker_id: assignee.id,
            description: req.description.clone(),
            description_multimedia: req.description_multimedia.clone(),
            assignment_date: Utc::now().naive_utc(),
            due_date: req.due_date,
            status_id,
            title: req.title.clone(),
            importance_id,
            category_id,
        };

        diesel::insert_into(tasks_table::tasks)
            .values(&new_task)
            .execute(conn)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Task created successfully")),
        Err(DieselError::NotFound) => {
            HttpResponse::BadRequest().json(Res::new("Workspace not found"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            HttpResponse::Conflict().json(Res::new("Task with this title already exists"))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
            HttpResponse::BadRequest()
                .json(Res::new("Invalid reference (workspace, user, or category)"))
        }
        Err(err) => {
            eprintln!("Error creating task: {}", err);
            HttpResponse::InternalServerError().json(Res::new("Server error while creating task"))
        }
    }
}
