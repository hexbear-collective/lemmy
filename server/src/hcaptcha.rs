use chrono::Utc;
use lemmy_utils::settings::Settings;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HCaptchaResponse {
  pub success: bool,
  pub challenge_ts: chrono::DateTime<Utc>,
  pub hostname: String,
  pub credit: bool,
  #[serde(rename = "error-codes")]
  pub error_codes: Option<Vec<String>>,
}

#[derive(Debug)]
pub enum ErrorCode {
  // hCaptia API Error Codes: (https://docs.hcaptcha.com/)
  MissingInputSecret,           // Your secret key is missing.
  InvalidInputSecret,           // Your secret key is invalid or malformed.
  MissingInputResponse,         // The response parameter (verification token) is missing.
  InvalidInputResponse,         // The response parameter (verification token) is invalid or malformed.
  BadRequest,                   // The request is invalid or malformed.
  InvalidOrAlreadySeenResponse, // The response parameter has already been checked, or has another issue.
  SitekeySecretMismatch,        // The sitekey is not registered with the provided secret.

  // custom error codes
  Unknown,       // API error code not recognized
  ParseError,    // Unable to parse response from hCaptcha
  RequestFailed, // Request failed for some reason
}

impl ErrorCode {
  fn from_str(code: &str) -> Self {
    match code {
      "missing-input-secret" => Self::MissingInputSecret,
      "invalid-input-secret" => Self::InvalidInputSecret,
      "missing-input-response" => Self::MissingInputResponse,
      "invalid-input-response" => Self::InvalidInputResponse,
      "bad-request" => Self::BadRequest,
      "invalid-or-already-seen-response" => Self::InvalidOrAlreadySeenResponse,
      "sitekey-secret-mismatch" => Self::SitekeySecretMismatch,
      _ => Self::Unknown,
    }
  }
}

#[derive(Debug)]
pub struct HCaptchaError {
  pub error_codes: Vec<ErrorCode>,
}

impl HCaptchaError {
  fn from_strings(codes: Vec<String>) -> Self {
    let mut ret = HCaptchaError {
      error_codes: Vec::new(),
    };
    for code in codes {
      ret.error_codes.push(ErrorCode::from_str(code.as_str()));
    }
    ret
  }

  fn err(code: ErrorCode) -> Self {
    HCaptchaError {
      error_codes: vec![code],
    }
  }
}

pub async fn hcaptcha_verify(hcaptcha_id: String) -> Result<HCaptchaResponse, HCaptchaError> {
  let client = reqwest::Client::new();
  let req_body = [
    ("secret", Settings::get().hcaptcha.secret_key),
    ("response", hcaptcha_id.clone()),
  ];

  let response = client
    .post(Settings::get().hcaptcha.verify_url.as_str())
    .form(&req_body)
    .send()
    .await;

  match response {
    Ok(response) => {
      match response.json::<HCaptchaResponse>().await {
        Ok(response) => {
          if response.success {
            Ok(response)
          } else if let Some(error_codes) = response.error_codes {
            Err(HCaptchaError::from_strings(error_codes))
          } else {
            Err(HCaptchaError::err(ErrorCode::Unknown))
          }
        }
        Err(e) => {
          error!("hCaptcha parse failed: {}", e);
          Err(HCaptchaError::err(ErrorCode::ParseError))
        }
      }
    }
    Err(e) => {
      error!("hCaptcha request failed: {}", e);
      Err(HCaptchaError::err(ErrorCode::RequestFailed))
    }
  }
}
