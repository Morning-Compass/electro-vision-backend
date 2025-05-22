use crate::buisness_logic::problems::db_problems::DbProblem;
use crate::est_conn;
use crate::multimedia_handler::{MultimediaHandler, MultimediaHandlerError};
use crate::response::Response as Res;
use crate::DPool;
use actix_web::post;
use actix_web::{
    web::Path,
    HttpResponse,
};
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

#[post("/workspace/{id}/problems/list")]
pub async fn list_problems(pool: DPool, path: Path<WorkspaceId>) -> HttpResponse {
    let workspace_id = path.id;

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
                let (multimedia, filename) = match problem.problem_multimedia_path {
                    Some(ref path) => match MultimediaHandler::get_file_content_base64(path) {
                        Ok(content) => {
                            let filename = std::path::Path::new(path)
                                .file_name()
                                .and_then(|f| f.to_str())
                                .map(|s| s.to_string());
                            (Some(content), filename)
                        }
                        Err(e) => {
                            return match e {
                                MultimediaHandlerError::DecodingError => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Decoding error"))
                                }
                                MultimediaHandlerError::InvalidFileType => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Invalid file type"))
                                }
                                MultimediaHandlerError::MaximumFileSizeReached => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("Maximum file size reached"))
                                }
                                MultimediaHandlerError::FileSystemError => {
                                    HttpResponse::InternalServerError()
                                        .json(Res::new("File system error"))
                                }
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

            HttpResponse::Ok().json(Res::new(res))
        }
        Err(DieselError::NotFound) => HttpResponse::NotFound().json(Res::new("No problems found")),
        Err(err) => {
            eprintln!("Error listing problems: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while listing problems"))
        }
    }
}
