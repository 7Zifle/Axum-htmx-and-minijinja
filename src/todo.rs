use serde::Serialize;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    FromRow, PgPool,
};
use uuid::Uuid;

#[derive(Serialize, Clone, Debug, FromRow)]
pub struct Todo {
    pub uuid: Uuid,
    pub name: String,
}

impl Todo {
    pub async fn db_insert(&self, pool: &PgPool) -> Result<PgRow, sqlx::Error> {
        sqlx::query!(
            "INSERT INTO todos(uuid, name) VALUES ($1, $2)",
            self.uuid,
            self.name
        )
        .fetch_one(pool)
        .await
    }

    pub async fn db_update(&self, pool: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "UPDATE todos SET name=$1 WHERE uuid=$2",
            self.name,
            self.uuid
        )
        .execute(pool)
        .await
    }

    // Class functions
    pub async fn db_get_todos(pool: &PgPool) -> Result<Vec<Todo>, sqlx::Error> {
        sqlx::query_as::<_, Todo>("SELECT uuid, name FROM todos")
            .fetch_all(pool)
            .await
    }

    pub async fn db_find_by_uuid(uuid: Uuid, pool: &PgPool) -> Result<Todo, sqlx::Error> {
        sqlx::query_as::<_, Todo>("SELECT uuid, name FROM todos WHERE uuid=$1")
            .bind(uuid)
            .fetch_one(pool)
            .await
    }

    pub async fn db_delete(uuid: Uuid, pool: &PgPool) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!("DELETE FROM todos WHERE uuid=$1", uuid)
            .execute(pool)
            .await
    }
}
