use rocket_sync_db_pools::{database};
#[database("mysql_path")] //look in rocket.toml to find the path to the db
pub struct DbConn(diesel::MysqlConnection);