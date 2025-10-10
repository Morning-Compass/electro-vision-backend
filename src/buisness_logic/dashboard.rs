use crate::{auth::find_user::Find, est_conn, response::Response as Res, DPool};
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct DashboardRequest {
    pub owner_email: String,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    pub workspace_stats: WorkspaceStats,
    pub task_stats: TaskStats,
    pub worker_stats: WorkerStats,
    pub nationality_data: Vec<NationalityData>,
}

#[derive(Serialize)]
pub struct WorkspaceStats {
    pub total: i64,
    pub active: i64,
    pub completed: i64,
}

#[derive(Serialize)]
pub struct TaskStats {
    pub total: i64,
    pub completed: i64,
    pub in_progress: i64,
    pub overdue: i64,
}

#[derive(Serialize)]
pub struct WorkerStats {
    pub total: i64,
    pub active: i64,
    pub by_position: Vec<PositionCount>,
}

#[derive(Serialize)]
pub struct PositionCount {
    pub position: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct NationalityData {
    pub country: String,
    pub count: i64,
    pub percentage: i64,
}

#[post("/workspace/dashboard")]
pub async fn dashboard(pool: DPool, req: Json<DashboardRequest>) -> HttpResponse {
    let owner_email = &req.owner_email;

    let workspaces = match crate::auth::find_user::FindData::find_workspace_by_owner_email(
        owner_email.clone(),
        pool.clone(),
    )
    .await
    {
        Ok(ws) => ws,
        Err(_) => {
            return HttpResponse::BadRequest().json(Res::new("No workspaces found for owner"))
        }
    };

    let workspace_ids: Vec<i32> = workspaces.iter().map(|w| w.id).collect();
    if workspace_ids.is_empty() {
        return HttpResponse::Ok().json(Res::new(DashboardResponse {
            workspace_stats: WorkspaceStats {
                total: 0,
                active: 0,
                completed: 0,
            },
            task_stats: TaskStats {
                total: 0,
                completed: 0,
                in_progress: 0,
                overdue: 0,
            },
            worker_stats: WorkerStats {
                total: 0,
                active: 0,
                by_position: vec![],
            },
            nationality_data: vec![],
        }));
    }

    let conn = &mut est_conn(pool);

    use crate::schema::{
        countries::dsl as c_dsl, full_users::dsl as fu_dsl, positions::dsl as pos_dsl,
        tasks::dsl as task_dsl, users_citizenships::dsl as uc_dsl, workspace_users::dsl as wu_dsl,
        workspaces::dsl as ws_dsl,
    };

    let now = chrono::Utc::now().naive_utc();

    let workspace_stats = {
        let total = workspace_ids.len() as i64;
        let active = ws_dsl::workspaces
            .filter(ws_dsl::id.eq_any(&workspace_ids))
            .filter(
                ws_dsl::finish_date
                    .is_null()
                    .or(ws_dsl::finish_date.gt(now)),
            )
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        let completed = total - active;
        WorkspaceStats {
            total,
            active,
            completed,
        }
    };

    let task_stats = {
        let total = task_dsl::tasks
            .filter(task_dsl::workspace_id.eq_any(&workspace_ids))
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        use crate::constants::status;

        let completed = task_dsl::tasks
            .filter(task_dsl::workspace_id.eq_any(&workspace_ids))
            .filter(task_dsl::status_id.eq(status::COMPLETED))
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        let in_progress = task_dsl::tasks
            .filter(task_dsl::workspace_id.eq_any(&workspace_ids))
            .filter(task_dsl::status_id.eq(status::IN_PROGRESS))
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        let overdue = task_dsl::tasks
            .filter(task_dsl::workspace_id.eq_any(&workspace_ids))
            .filter(task_dsl::due_date.lt(now))
            .filter(task_dsl::status_id.ne(status::COMPLETED))
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        TaskStats {
            total,
            completed,
            in_progress,
            overdue,
        }
    };

    let worker_stats = {
        let total = wu_dsl::workspace_users
            .filter(wu_dsl::workspace_id.eq_any(&workspace_ids))
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        let active = total;

        // Get positions with counts for each workspace
        let by_position = pos_dsl::positions
            .inner_join(
                wu_dsl::workspace_users.on(wu_dsl::position_id
                    .eq(pos_dsl::id.nullable())
                    .and(wu_dsl::workspace_id.eq_any(&workspace_ids))),
            )
            .filter(pos_dsl::workspace_id.eq_any(&workspace_ids))
            .group_by(pos_dsl::name)
            .select((pos_dsl::name.nullable(), diesel::dsl::count_star())) // Make name nullable
            .load::<(Option<String>, i64)>(conn) // Load as Option<String> to handle NULL
            .unwrap_or_default()
            .into_iter()
            .map(|(pos, count)| PositionCount {
                position: pos.unwrap_or_else(|| "Not Assigned".to_string()), // Handle NULL names
                count,
            })
            .collect::<Vec<_>>();

        // Get count of users with NULL position
        let null_position_count = wu_dsl::workspace_users
            .filter(wu_dsl::workspace_id.eq_any(&workspace_ids))
            .filter(wu_dsl::position_id.is_null())
            .count()
            .get_result::<i64>(conn)
            .unwrap_or(0);

        // Combine results
        let mut all_positions = by_position;

        // Add "Not Assigned" count if there are any
        if null_position_count > 0 {
            all_positions.push(PositionCount {
                position: "Not Assigned".to_string(),
                count: null_position_count,
            });
        }

        WorkerStats {
            total,
            active,
            by_position: all_positions,
        }
    };

    let nationality_data = {
        // 1. Get all unique user_ids associated with the workspace_ids
        let user_ids_in_workspaces: Vec<i32> = wu_dsl::workspace_users
            .filter(wu_dsl::workspace_id.eq_any(&workspace_ids))
            .select(wu_dsl::user_id)
            .distinct()
            .load::<i32>(conn)
            .unwrap_or_default();

        // 2. Fetch country_of_origin for these users from full_users
        //    Group by country name and count directly in the query.
        let raw_results = fu_dsl::full_users
            .inner_join(c_dsl::countries.on(c_dsl::id.eq(fu_dsl::country_of_origin_id)))
            .filter(fu_dsl::user_id.eq_any(&user_ids_in_workspaces))
            .group_by(c_dsl::name)
            .select((c_dsl::name.nullable(), diesel::dsl::count_star()))
            .load::<(Option<String>, i64)>(conn)
            .unwrap_or_default();

        // 3. Calculate total count of users with origin country data
        let total_origin_users: i64 = raw_results.iter().map(|(_, count)| *count).sum();

        // 4. Map to NationalityData struct
        raw_results
            .into_iter()
            .map(|(country_name_opt, count)| {
                let country_name = country_name_opt.unwrap_or_else(|| "Unknown Origin".to_string());
                let percentage = if total_origin_users > 0 {
                    (count * 100) / total_origin_users
                } else {
                    0
                };
                NationalityData {
                    country: country_name,
                    count,
                    percentage,
                }
            })
            .collect::<Vec<_>>()
    };

    let response = DashboardResponse {
        workspace_stats,
        task_stats,
        worker_stats,
        nationality_data,
    };

    HttpResponse::Ok().json(Res::new(response))
}
