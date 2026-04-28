use super::schema::history;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct History {
    pub id: Option<i32>,
    pub connection_id: i32,
    pub started_at: i32,
    pub ended_at: i32,
    pub exit_code: String,
}

#[derive(Insertable)]
#[diesel(table_name = history)]
pub struct NewHistory {
    pub connection_id: i32,
    pub started_at: i32,
    pub ended_at: i32,
    pub exit_code: String,
}

impl History {
    pub fn create(
        db_connection: &mut SqliteConnection,
        connection_id: i32,
        started_at: i32,
        ended_at: i32,
        exit_code: String,
    ) -> QueryResult<usize> {
        let new_history = NewHistory {
            connection_id,
            started_at,
            ended_at,
            exit_code,
        };

        diesel::insert_into(history::table)
            .values(&new_history)
            .execute(db_connection)
    }

    pub fn get_recent(db_connection: &mut SqliteConnection, limit: i64) -> QueryResult<Vec<History>> {
        history::table
            .order(history::started_at.desc())
            .limit(limit)
            .load(db_connection)
    }

    pub fn get_by_connection(
        db_connection: &mut SqliteConnection,
        conn_id: i32,
        limit: i64,
    ) -> QueryResult<Vec<History>> {
        history::table
            .filter(history::connection_id.eq(conn_id))
            .order(history::started_at.desc())
            .limit(limit)
            .load(db_connection)
    }
}
