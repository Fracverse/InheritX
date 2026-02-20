use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::api_error::ApiError;

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct AuthAdmin {
    pub admin_id: Uuid,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(user_id) = parse_bearer_uuid(parts) {
            return Ok(Self { user_id });
        }

        let user_id = parts
            .headers
            .get("x-user-id")
            .ok_or(ApiError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?;

        let user_id = Uuid::parse_str(user_id).map_err(|_| ApiError::Unauthorized)?;

        Ok(Self { user_id })
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthAdmin
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let admin_id = parts
            .headers
            .get("x-admin-id")
            .ok_or(ApiError::Unauthorized)?
            .to_str()
            .map_err(|_| ApiError::Unauthorized)?;

        let admin_id = Uuid::parse_str(admin_id).map_err(|_| ApiError::Unauthorized)?;

        Ok(Self { admin_id })
    }
}

fn parse_bearer_uuid(parts: &Parts) -> Option<Uuid> {
    let header = parts.headers.get(AUTHORIZATION)?;
    let token = header.to_str().ok()?;
    let token = token.strip_prefix("Bearer ")?;

    Uuid::parse_str(token).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::FromRequestParts;
    use axum::http::Request;

    #[tokio::test]
    async fn auth_user_accepts_x_user_id_header() {
        let user_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/plans")
            .header("x-user-id", user_id.to_string())
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_user = AuthUser::from_request_parts(&mut parts, &())
            .await
            .expect("x-user-id should authenticate user");

        assert_eq!(auth_user.user_id, user_id);
    }

    #[tokio::test]
    async fn auth_user_accepts_bearer_uuid_token() {
        let user_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/plans")
            .header(AUTHORIZATION, format!("Bearer {}", user_id))
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_user = AuthUser::from_request_parts(&mut parts, &())
            .await
            .expect("bearer uuid should authenticate user");

        assert_eq!(auth_user.user_id, user_id);
    }

    #[tokio::test]
    async fn auth_user_rejects_missing_headers() {
        let req = Request::builder()
            .uri("/plans")
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let result = AuthUser::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn auth_admin_accepts_x_admin_id_header() {
        let admin_id = Uuid::new_v4();
        let req = Request::builder()
            .uri("/admin/plans")
            .header("x-admin-id", admin_id.to_string())
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let auth_admin = AuthAdmin::from_request_parts(&mut parts, &())
            .await
            .expect("x-admin-id should authenticate admin");

        assert_eq!(auth_admin.admin_id, admin_id);
    }

    #[tokio::test]
    async fn auth_admin_rejects_missing_headers() {
        let req = Request::builder()
            .uri("/admin/plans")
            .body(())
            .expect("request should build");
        let (mut parts, _) = req.into_parts();

        let result = AuthAdmin::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(ApiError::Unauthorized)));
    }
}
