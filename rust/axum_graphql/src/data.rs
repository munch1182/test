use std::{thread::sleep, time::Duration};

use async_graphql::{Schema, SimpleObject};
use futures_util::Stream;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Serialize, Deserialize, Clone)]
pub struct User {
    id: u32,
    name: String,
    email: String,
}

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn ping(&self) -> &str {
        "pong"
    }

    async fn users(&self) -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ]
    }

    async fn user(&self, id: u32) -> Option<User> {
        match id {
            1 => Some(User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            }),
            2 => Some(User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            }),
            _ => None,
        }
    }
}

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn create_user(&self, name: String, email: String) -> User {
        User { id: 3, name, email }
    }
    // 更新用户
    async fn update_user(&self, id: u32, name: String, email: String) -> Option<User> {
        Some(User { id, name, email })
    }
}

pub struct SubscriptionRoot;

#[async_graphql::Subscription]
impl SubscriptionRoot {
    async fn counter(&self, from: u32) -> impl Stream<Item = u32> {
        let mut value = from;
        async_stream::stream! {
            loop {
                sleep(Duration::from_secs(1));
                yield value;
                value += 1;
            }
        }
    }

    // 用户创建事件订阅
    async fn user_created(&self) -> impl Stream<Item = User> {
        let users = vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ];
        async_stream::stream! {
            for user in users {
                yield user.clone();
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

pub type MySchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema() -> MySchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot).finish()
}
