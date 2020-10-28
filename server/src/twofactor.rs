use lemmy_api_structs::APIError;
use lemmy_db::user::*;
use lemmy_utils::{send_email, settings::Settings, LemmyError};

use std::{sync::Mutex, time::Duration};

use chrono::prelude::*;
use rand::seq::IteratorRandom;
use ttl_cache::TtlCache;

pub struct CodeCacheHandler {
  cache: Mutex<TtlCache<String, User_>>,
}

impl CodeCacheHandler {
  pub fn new() -> CodeCacheHandler {
    CodeCacheHandler {
      cache: Mutex::new(TtlCache::new(Settings::get().twofactor.cache_size)),
    }
  }

  pub fn generate_2fa(&self, user: User_) -> Result<(), LemmyError> {
    if !user.has_2fa || user.email.is_none() {
      return Err(APIError::err("user_has_no_2fa").into());
    }

    let mut genned_code;
    {
      //inner scope to unlock code cache once we write the code
      let mut code_cache = match self.cache.lock() {
        Ok(k) => k,
        Err(_e) => return Err(APIError::err("internal_error").into()),
      };
      let mut rng = rand::thread_rng();
      let config = Settings::get().twofactor;
      loop {
        genned_code = String::from("");
        for _ in 0..config.code_length {
          genned_code.push(config.allowed_characters.chars().choose(&mut rng).unwrap());
        }
        if code_cache.get(genned_code.as_str()).is_none() {
          //break if this is a unique 2fa code
          break;
        }
      }

      code_cache.insert(genned_code.clone(), user.clone(), Duration::from_secs(3600));
      //code is valid for one hour
    }

    let subject = &format!("ChapoChat: Attempted login for {}", &user.name);
    let time = Utc::now().format("%Y-%m-%d %H:%M:%S");
    let html = &format!("<h1>Attempted login for {}</h1><br><p>At {} UTC a login was attempted on your account.
              Because your account is setup with two-factor authentication, you must enter a code to successfully login. This code will expire within one hour.</p>
              <h3>Your login code is {}</h3>", user.name, time, genned_code);
    //println!("Sending 2fa email with code {}", genned_code);
    match send_email(subject, user.email.unwrap().as_str(), &user.name, html) {
      Ok(_k) => (),
      Err(e) => println!("Failed to send email: {}", e),
    }
    Ok(())
  }
  pub fn check_2fa(&self, user: &User_, code: &str) -> Result<bool, LemmyError> {
    if !user.has_2fa || user.email.is_none() {
      return Err(APIError::err("user_has_no_2fa").into());
    }
    let mut code_cache = match self.cache.lock() {
      Ok(k) => k,
      Err(_e) => return Err(APIError::err("internal_error").into()),
    };
    //println!("Entered code {}", code);
    match code_cache.get(code) {
      Some(cached_user) => {
        if cached_user.id == user.id {
          code_cache.remove(code);
          return Ok(true); //code exists and the user did request it
        }
        Ok(false) //the code exists, but the user wasn't the one who requested it
      }
      None => Ok(false), //no matching code
    }
  }
}

impl Default for CodeCacheHandler {
  fn default() -> Self {
    CodeCacheHandler::new()
  }
}
