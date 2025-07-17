use axum::{
    body::Body, extract::Request, http::StatusCode, middleware::Next, response::{Json, Response}
};
use dotenv::dotenv;
use std::env;
pub mod types;

use jsonwebtoken::{
 decode,Validation,DecodingKey
};
use types::Claims;

pub async fn auth_middlware(req:Request<Body>,next:Next)->Result<Response,StatusCode>{


    dotenv().ok();
  let header=req.headers();
  if let Some(auth_header) =header.get("Authorization") {
    if let Ok(auth_str)=auth_header.to_str(){
    if auth_str.starts_with("Bearer ") {
        let token = &auth_str[7..];
        let secret = env::var("JWT_SECRATE").expect("value not loading");
        let decode_result = decode::<Claims>(
          token,
          &DecodingKey::from_secret(secret.as_bytes()),
          &Validation::default(),
        );

        match decode_result {
          Ok(token_data) => {
            // Token is valid
            let response = next.run(req).await;
            return Ok(response);
          }
          Err(_) => {
            // Token is invalid or tampered
            return Err(StatusCode::UNAUTHORIZED);
          }
        }
    }
    }
  }
  Err(StatusCode::UNAUTHORIZED)
}

