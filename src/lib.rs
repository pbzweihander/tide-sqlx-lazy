use sqlx_core::database::Database;
use sqlx_core::pool::{Pool, PoolConnection};
use tide::{Middleware, Next, Request};

#[cfg(feature = "tracing")]
use libtracing::{info_span, Instrument};

pub struct SqlxMiddleware<DB>
where
    DB: Database,
{
    pool: Pool<DB>,
}

impl<DB> SqlxMiddleware<DB>
where
    DB: Database,
{
    pub fn new(pool: Pool<DB>) -> Self {
        Self { pool }
    }
}

impl<DB> From<Pool<DB>> for SqlxMiddleware<DB>
where
    DB: Database,
{
    fn from(pool: Pool<DB>) -> Self {
        Self::new(pool)
    }
}

impl<DB> Into<Pool<DB>> for SqlxMiddleware<DB>
where
    DB: Database,
{
    fn into(self) -> Pool<DB> {
        self.pool
    }
}

impl<DB> AsRef<Pool<DB>> for SqlxMiddleware<DB>
where
    DB: Database,
{
    fn as_ref(&self) -> &Pool<DB> {
        &self.pool
    }
}

#[async_trait::async_trait]
impl<State, DB> Middleware<State> for SqlxMiddleware<DB>
where
    State: Clone + Send + Sync + 'static,
    DB: Database,
{
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> tide::Result {
        req.set_ext(self.pool.clone());
        Ok(next.run(req).await)
    }
}

#[async_trait::async_trait]
pub trait SqlxRequestExt {
    fn sqlx_pool<DB>(&self) -> &Pool<DB>
    where
        DB: Database;

    async fn sqlx_conn<DB>(&self) -> Result<PoolConnection<DB>, sqlx_core::error::Error>
    where
        DB: Database,
    {
        let acquire_fut = self.sqlx_pool().acquire();
        #[cfg(feature = "tracing")]
        let acquire_fut = acquire_fut.instrument(info_span!("Acquiring database connection"));
        acquire_fut.await
    }
}

#[async_trait::async_trait]
impl<State> SqlxRequestExt for Request<State> {
    fn sqlx_pool<DB>(&self) -> &Pool<DB>
    where
        DB: Database,
    {
        self.ext()
            .expect("You must install SqlxMiddleware providing sqlx Pool")
    }
}
