use super::schema::connection_hops;
use super::connection::Connection;
use diesel::prelude::*;
use diesel::Connection as _;

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = connection_hops)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ConnectionHop {
    pub source_connection_id: i32,
    pub target_connection_id: i32,
    pub hop_order: i32,
}

impl ConnectionHop {
    pub fn get_jumps(
        db_connection: &mut SqliteConnection,
        source_id: i32,
    ) -> QueryResult<Vec<Connection>> {
        use super::schema::{connection_hops, connections};
        
        connection_hops::table
            .filter(connection_hops::source_connection_id.eq(source_id))
            .inner_join(connections::table.on(connection_hops::target_connection_id.nullable().eq(connections::id)))
            .order(connection_hops::hop_order.asc())
            .select(Connection::as_select())
            .load::<Connection>(db_connection)
    }

    pub fn set_jumps(
        db_connection: &mut SqliteConnection,
        source_id: i32,
        target_ids: Vec<i32>,
    ) -> QueryResult<()> {
        db_connection.transaction(|conn| {
            diesel::delete(connection_hops::table.filter(connection_hops::source_connection_id.eq(source_id)))
                .execute(conn)?;
                
            let new_hops: Vec<ConnectionHop> = target_ids
                .into_iter()
                .enumerate()
                .map(|(index, target_id)| ConnectionHop {
                    source_connection_id: source_id,
                    target_connection_id: target_id,
                    hop_order: index as i32,
                })
                .collect();
                
            diesel::insert_into(connection_hops::table)
                .values(&new_hops)
                .execute(conn)?;
                
            Ok(())
        })
    }

    pub fn get_all_jump_target_ids(
        db_connection: &mut SqliteConnection,
    ) -> QueryResult<std::collections::HashSet<i32>> {
        let ids: Vec<i32> = connection_hops::table
            .select(connection_hops::target_connection_id)
            .distinct()
            .load(db_connection)?;
        Ok(ids.into_iter().collect())
    }

    pub fn get_all_proxy_destination_ids(
        db_connection: &mut SqliteConnection,
    ) -> QueryResult<std::collections::HashSet<i32>> {
        let ids: Vec<i32> = connection_hops::table
            .select(connection_hops::source_connection_id)
            .distinct()
            .load(db_connection)?;
            
        Ok(ids.into_iter().collect())
    }
}
