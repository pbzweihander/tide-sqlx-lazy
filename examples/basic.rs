use tide_sqlx_lazy::SqlxMiddleware;

type DatabaseType = sqlx::Postgres;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let pool = sqlx::Pool::<DatabaseType>::connect_lazy(&std::env::var("DATABASE_URL").unwrap())?;

    let middleware = SqlxMiddleware::new(pool);

    let mut server = tide::new();

    server.with(middleware);

    server.at("/with_db").get(with_db);
    server.at("/without_db").get(without_db);

    server.listen("0.0.0.0:8080").await?;

    Ok(())
}

async fn with_db(req: tide::Request<()>) -> tide::Result {
    use tide_sqlx_lazy::SqlxRequestExt;

    let mut db_conn = req.sqlx_conn().await?;

    let query_resp: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&mut db_conn)
        .await?;
    assert_eq!(query_resp, 1);

    Ok(tide::Response::new(tide::StatusCode::Ok))
}

async fn without_db(_: tide::Request<()>) -> tide::Result {
    // No DB connection occurred
    Ok(tide::Response::new(tide::StatusCode::Ok))
}
