use diesel::{Connection, PgConnection, RunQueryDsl};

pub fn clear_db(db_url: &str, db_name: &str) {
    let mut conn = PgConnection::establish(&db_url).expect("Cannot connect to postgres database.");

    let disconnect_users = format!(
        "SELECT pg_terminate_backend(pid)
            FROM pg_stat_activity
            WHERE datname = '{}';",
        db_name
    );

    RunQueryDsl::execute(diesel::sql_query(disconnect_users.as_str()), &mut conn).unwrap();

    let query = diesel::sql_query(format!("DROP DATABASE IF EXISTS {}", db_name).as_str());
    RunQueryDsl::execute(query, &mut conn)
        .unwrap_or_else(|e| panic!("Couldn't drop database {}, error: {}", db_name, e.to_string()));
}
