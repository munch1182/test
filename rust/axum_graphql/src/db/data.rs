use async_graphql::{Context, InputObject, Object, SimpleObject};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, prelude::FromRow, query, query_as};

use crate::db::{SqlResult, datac::PaginationInput};

#[derive(SimpleObject, Serialize, Deserialize, Clone, FromRow, Debug)]
pub struct User {
    id: i64,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[derive(InputObject, Debug)]
pub struct CreateUserInput {
    pub name: String,
    pub email: String,
}

pub struct QueryRoot;
pub struct MutationRoot;

#[Object]
impl QueryRoot {
    ///
    /// GraphQL标准中没有无符号整数类型
    /// 如果传入负数，SQLite会将其视为0
    ///
    async fn users(&self, ctx: &Context<'_>, p: Option<PaginationInput>) -> SqlResult<Vec<User>> {
        let pool = ctx.data::<SqlitePool>()?;
        let p = p.unwrap_or_default();
        let limit = p.limit();
        let skip = p.skip();

        let users = query_as!(
            User,
            "SELECT id, name, email, created_at as \"created_at: _\" FROM users LIMIT ? OFFSET ?",
            limit,
            skip
        )
        .fetch_all(pool)
        .await?;
        Ok(users)
    }

    async fn user(&self, ctx: &Context<'_>, id: u32) -> SqlResult<Option<User>> {
        let pool = ctx.data::<SqlitePool>()?;
        let user =
            sqlx::query_as::<_, User>("SELECT id, name, email, created_at FROM users WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await?;
        Ok(user)
    }
}

#[Object]
impl MutationRoot {
    async fn crate_user(&self, ctx: &Context<'_>, input: CreateUserInput) -> SqlResult<u32> {
        let pool = ctx.data::<SqlitePool>()?;
        let now = Utc::now();

        let result = query!(
            "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?) RETURNING id",
            input.name,
            input.email,
            now
        )
        .fetch_one(pool)
        .await?;

        Ok(result.id as u32)
    }
}
