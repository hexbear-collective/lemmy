use lemmy_api_structs::APIError;
use lemmy_db::user::*;
use lemmy_utils::{send_email, LemmyError};

use std::{sync::Mutex, time::Duration};

use chrono::prelude::*;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use ttl_cache::TtlCache;

lazy_static! { //no real good way to synchronously pass the cache between all login requests without an ugly static with inner mutability
    static ref CODE_CACHE: Mutex<TtlCache<String, User_>> = Mutex::new(TtlCache::new(100)); //making me choose a max number of items
}

const ALLOWED_CODE_CHARS: &[u8] = b"0123456789";
const CODE_LENGTH: usize = 8;

pub fn generate_2fa(user: User_) -> Result<(), LemmyError> {
  if !user.has_2fa || user.email.is_none() {
    return Err(APIError::err("user_has_no_2fa").into());
  }

  let mut genned_code;
  {
    //inner scope to unlock code cache once we write the code
    let mut code_cache = match CODE_CACHE.lock() {
      Ok(k) => k,
      Err(_e) => return Err(APIError::err("internal_error").into()),
    };
    let mut rng = rand::thread_rng();
    loop {
      genned_code = String::from("");
      for _ in 0..=CODE_LENGTH {
        genned_code.push(
          ALLOWED_CODE_CHARS
            .choose(&mut rng)
            .map(|&c| c as char)
            .unwrap(),
        );
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

pub fn check_2fa(user: &User_, code: &String) -> Result<bool, LemmyError> {
  if !user.has_2fa || user.email.is_none() {
    return Err(APIError::err("user_has_no_2fa").into());
  }
  let mut code_cache = match CODE_CACHE.lock() {
    Ok(k) => k,
    Err(_e) => return Err(APIError::err("internal_error").into()),
  };
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
