use actix_web::{dev::Payload, Error, FromRequest, HttpRequest, HttpMessage, HttpResponse};
use futures::future::{ready, Ready};
use crate::auth::jwt::{decode_jwt, Claims};
use uuid::Uuid;

pub struct AuthenticatedUser {
    pub id: Uuid,
    pub email: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<AuthenticatedUser, Error>>;
    // No config
    // type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // read Authorization header
        let header = req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .and_then(|hv| hv.to_str().ok())
            .unwrap_or("");

        if !header.starts_with("Bearer ") {
            return ready(Err(actix_web::error::ErrorUnauthorized("Missing or malformed auth header")));
        }

        let token = &header[7..];

        match decode_jwt(token) {
            Ok(token_data) => {
                let claims: Claims = token_data.claims;
                ready(Ok(AuthenticatedUser {
                    id: claims.sub,
                    email: claims.email,
                }))
            }
            Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
        }
    }
}
