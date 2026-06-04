use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::HeaderMap;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::auth::UserClaims;
use crate::config::Config;

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
        let user = ctx
            .data::<UserClaims>()
            .map_err(|_| async_graphql::Error::new("Authentication required"))?;
        let parsed_id =
            Uuid::parse_str(&id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        let record =
            sqlx::query("SELECT id, user_id, status FROM plans WHERE id = $1 AND user_id = $2")
                .bind(parsed_id)
                .bind(user.user_id)
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
        let caller = ctx
            .data::<UserClaims>()
            .map_err(|_| async_graphql::Error::new("Authentication required"))?;
        let parsed_id =
            Uuid::parse_str(&user_id).map_err(|_| async_graphql::Error::new("Invalid ID"))?;

        if parsed_id != caller.user_id {
            return Ok(None);
        }

        let record = sqlx::query(
            "SELECT user_id, score, total_loans_taken, total_loans_repaid FROM user_reputation WHERE user_id = $1",
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

pub fn create_schema(db: PgPool, config: Config) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(db)
        .data(config)
        .finish()
}

fn extract_user_claims(headers: &HeaderMap, config: &Config) -> Option<UserClaims> {
    if let Some(auth_header) = headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if let Ok(decoded) = jsonwebtoken::decode::<UserClaims>(
                token,
                &jsonwebtoken::DecodingKey::from_secret(config.jwt_secret.as_bytes()),
                &jsonwebtoken::Validation::default(),
            ) {
                return Some(decoded.claims);
            }
        }
    }

    headers
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .and_then(|user_id_str| Uuid::parse_str(user_id_str).ok())
        .map(|user_id| UserClaims {
            user_id,
            email: "legacy-test@example.com".to_string(),
            exp: 0,
        })
}

pub async fn graphql_handler(
    State(schema): State<AppSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let config = match schema.data::<Config>() {
        Some(config) => config.clone(),
        None => {
            return GraphQLResponse(async_graphql::BatchResponse::Single(
                async_graphql::Response::from_errors(vec![async_graphql::ServerError::new(
                    "Server configuration unavailable",
                    None,
                )]),
            ))
        }
    };

    let Some(user) = extract_user_claims(&headers, &config) else {
        return GraphQLResponse(async_graphql::BatchResponse::Single(
            async_graphql::Response::from_errors(vec![async_graphql::ServerError::new(
                "Authentication required",
                None,
            )]),
        ));
    };

    let mut gql_req = req.into_inner();
    gql_req = gql_req.data(user);
    schema.execute(gql_req).await.into()
}

pub async fn graphql_playground() -> axum::response::Html<String> {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/api/graphql"),
    ))
}
