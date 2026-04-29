use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct Plan {
    id: String,
    user_id: String,
    status: String,
}

#[derive(SimpleObject)]
pub struct UserReputation {
    user_id: String,
    score: i32,
    total_loans_taken: i32,
    total_loans_repaid: i32,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn plan(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<Plan>> {
        let db = ctx
            .data::<PgPool>()
            .map_err(|_| async_graphql::Error::new("DB not found"))?;
        let parsed_id =
            Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let record = sqlx::query("SELECT id, user_id, status FROM plans WHERE id = $1")
            .bind(parsed_id)
            .fetch_optional(db)
            .await?;

        Ok(record.map(|r| Plan {
            id: r.get::<Uuid, _>("id").to_string(),
            user_id: r.get::<Uuid, _>("user_id").to_string(),
            status: r.get::<String, _>("status"),
        }))
    }

    async fn reputation(
        &self,
        ctx: &Context<'_>,
        user_id: String,
    ) -> async_graphql::Result<Option<UserReputation>> {
        let db = ctx
            .data::<PgPool>()
            .map_err(|_| async_graphql::Error::new("DB not found"))?;
        let parsed_id =
            Uuid::parse_str(&user_id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let record = sqlx::query(
            "SELECT user_id, score, total_loans_taken, total_loans_repaid FROM user_reputation WHERE user_id = $1"
        )
        .bind(parsed_id)
        .fetch_optional(db)
        .await?;

        Ok(record.map(|r| UserReputation {
            user_id: r.get::<Uuid, _>("user_id").to_string(),
            score: r.get::<i32, _>("score"),
            total_loans_taken: r.get::<i32, _>("total_loans_taken"),
            total_loans_repaid: r.get::<i32, _>("total_loans_repaid"),
        }))
    }
}

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema(db: PgPool) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(db)
        .finish()
}

pub async fn graphql_handler(
    State(schema): State<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/api/graphql"),
    ))
}
