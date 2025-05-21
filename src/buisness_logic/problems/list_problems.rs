use crate::buisness_logic::problems::db_problems::DbProblem;
use crate::multimedia_handler::MultimediaHandler;
use crate::response::Response as Res;
use crate::DPool;
use crate::{est_conn, models_insertable};
use actix_web::post;
use actix_web::{
    web::{Json, Path},
    HttpResponse,
};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ProblemResponse {
    id: i32,
    description: Option<String>,
    problem_multimedia: Option<String>,
    problem_multimedia_filename: Option<String>,
    worker_username: String,
    mentor_username: String,
}

#[derive(Deserialize)]
pub struct WorkspaceId {
    id: i32,
}

#[derive(Deserialize)]
pub struct ListProblemsRequest {
    owner_email: String,
}

#[post("/workspace/{id}/problems/list")]
pub async fn list_problems(
    pool: DPool,
    path: Path<WorkspaceId>,
    req: Json<ListProblemsRequest>,
) -> HttpResponse {
    let workspace_id = path.id;

    // You might want to add a check here to ensure the owner_email has access to the workspace.
    // For now, we'll assume the workspace_id is valid based on the route.

    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, Error, _>(|c| {
        let query = r#"
            SELECT
                problems.id,
                problems.description,
                problems.problem_multimedia_path,
                worker.username as worker_username,
                mentor.username as mentor_username
            FROM problems
            JOIN auth_users worker ON problems.worker_id = worker.id
            JOIN auth_users mentor ON problems.mentor_id = mentor.id
            WHERE problems.workspace_id = $1
        "#;
        diesel::sql_query(query)
            .bind::<diesel::sql_types::Integer, _>(workspace_id)
            .load::<DbProblem>(c)
    });

    match result {
        Ok(problems) => {
            let mut res: Vec<ProblemResponse> = Vec::new();

            for problem in problems {
                let (multimedia, filename) = match problem.description_multimedia_path {
                    Some(ref path) => match MultimediaHandler::get_file_content_base64(path) {
                        Ok(content) => {
                            let filename = std::path::Path::new(path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(|s| s.to_string());
                            (Some(content), filename)
                        }
                        Err(e) => {
                            return match e.kind() {
                                std::io::ErrorKind::NotFound => {
                                    HttpResponse::NotFound().json(Res::new("File not found."))
                                }
                                std::io::ErrorKind::PermissionDenied => HttpResponse::Forbidden()
                                    .json(Res::new("Permission denied while accessing file.")),
                                _ => HttpResponse::InternalServerError()
                                    .json(Res::new("Failed to read multimedia file.")),
                            };
                        }
                    },
                    None => (None, None),
                };

                let problem_res = ProblemResponse {
                    id: problem.id,
                    description: problem.description,
                    problem_multimedia: multimedia,
                    problem_multimedia_filename: filename,
                    worker_username: problem.worker_username,
                    mentor_username: problem.mentor_username,
                };

                res.push(problem_res);
            }

            HttpResponse::Ok().json(res)
        }
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("No problems found")),
        Err(err) => {
            eprintln!("Error listing problems: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while listing problems"))
        }
    }
}
