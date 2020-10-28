use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

use lemmy_utils::settings::Settings;

type Jwt = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub id: i32,
  pub token_id: uuid::Uuid,
  pub iss: String,
}

impl Claims {
  pub fn decode(jwt: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let v = Validation {
      validate_exp: false,
      ..Validation::default()
    };
    decode::<Claims>(
      &jwt,
      &DecodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
      &v,
    )
  }

  pub fn jwt(user_id: i32, token_id: uuid::Uuid, hostname: String) -> Result<Jwt, jsonwebtoken::errors::Error> {
    let my_claims = Claims {
      id: user_id,
      token_id,
      iss: hostname,
    };
    encode(
      &Header::default(),
      &my_claims,
      &EncodingKey::from_secret(Settings::get().jwt_secret.as_ref()),
    )
  }
}
