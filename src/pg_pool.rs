use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;

pub fn create_pg_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    let db_config = PostgresConnectionManager::new(
        "host=rinha-db user=rinha-db password=rinha-db".parse().unwrap(),
        NoTls,
    );
    r2d2::Pool::new(db_config).unwrap()
}
