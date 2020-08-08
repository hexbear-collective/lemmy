#[macro_use]
pub extern crate lazy_static;
pub extern crate comrak;
pub extern crate lettre;
pub extern crate lettre_email;
pub extern crate openssl;
pub extern crate rand;
pub extern crate regex;
pub extern crate serde_json;
pub extern crate url;

pub mod settings;

use crate::settings::Settings;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Utc};
use itertools::Itertools;
use lettre::{
  smtp::{
    authentication::{Credentials, Mechanism},
    extension::ClientId,
    ConnectionReuseParameters,
  },
  ClientSecurity,
  SmtpClient,
  Transport,
};
use lettre_email::Email;
use openssl::{pkey::PKey, rsa::Rsa};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::{Regex, RegexBuilder};
use std::io::{Error, ErrorKind};
use url::Url;

pub fn to_datetime_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
  DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn naive_from_unix(time: i64) -> NaiveDateTime {
  NaiveDateTime::from_timestamp(time, 0)
}

pub fn convert_datetime(datetime: NaiveDateTime) -> DateTime<FixedOffset> {
  let now = Local::now();
  DateTime::<FixedOffset>::from_utc(datetime, *now.offset())
}

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub fn num_md_images(test: &str) -> i32 {
  MD_IMAGE_REGEX.find_iter(test).count() as i32
}

pub fn remove_slurs(test: &str) -> String {
  SLUR_REGEX.replace_all(test, "*removed*").to_string()
}

pub fn remove_pii(test: &str) -> String {
  PII_REGEX.replace_all(test, "*removed*").to_string()
  //TODO: add other pii filters.
}

pub fn pii_check(test: &str) -> Result<(), Vec<&str>> {
  let mut matches: Vec<&str> = PII_REGEX.find_iter(test).map(|mat| mat.as_str()).collect();

  matches.sort_unstable();
  matches.dedup();

  if matches.is_empty() {
    Ok(())
  } else {
    Err(matches)
  }
}

pub fn slur_check(test: &str) -> Result<(), Vec<&str>> {
  let mut matches: Vec<&str> = SLUR_REGEX.find_iter(test).map(|mat| mat.as_str()).collect();

  // Unique
  matches.sort_unstable();
  matches.dedup();

  if matches.is_empty() {
    Ok(())
  } else {
    Err(matches)
  }
}

pub fn slurs_vec_to_str(slurs: Vec<&str>) -> String {
  let start = "No slurs - ";
  let combined = &slurs.join(", ");
  [start, combined].concat()
}

pub fn pii_vec_to_str(pii: Vec<&str>) -> String {
  let start = "No personally identifiable information - ";
  let combined = &pii.join(", ");
  [start, combined].concat()
}

pub fn generate_random_string() -> String {
  thread_rng().sample_iter(&Alphanumeric).take(30).collect()
}

pub fn send_email(
  subject: &str,
  to_email: &str,
  to_username: &str,
  html: &str,
) -> Result<(), String> {
  let email_config = Settings::get().email.ok_or("no_email_setup")?;

  let email = Email::builder()
    .to((to_email, to_username))
    .from(email_config.smtp_from_address.to_owned())
    .subject(subject)
    .html(html)
    .build()
    .unwrap();

  let mailer = if email_config.use_tls {
    SmtpClient::new_simple(&email_config.smtp_server).unwrap()
  } else {
    SmtpClient::new(&email_config.smtp_server, ClientSecurity::None).unwrap()
  }
  .hello_name(ClientId::Domain(Settings::get().hostname))
  .smtp_utf8(true)
  .authentication_mechanism(Mechanism::Plain)
  .connection_reuse(ConnectionReuseParameters::ReuseUnlimited);
  let mailer = if let (Some(login), Some(password)) =
    (&email_config.smtp_login, &email_config.smtp_password)
  {
    mailer.credentials(Credentials::new(login.to_owned(), password.to_owned()))
  } else {
    mailer
  };

  let mut transport = mailer.transport();
  let result = transport.send(email.into());
  transport.close();

  match result {
    Ok(_) => Ok(()),
    Err(e) => Err(e.to_string()),
  }
}

pub fn markdown_to_html(text: &str) -> String {
  comrak::markdown_to_html(text, &comrak::ComrakOptions::default())
}

// TODO nothing is done with community / group webfingers yet, so just ignore those for now
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MentionData {
  pub name: String,
  pub domain: String,
}

impl MentionData {
  pub fn is_local(&self) -> bool {
    Settings::get().hostname.eq(&self.domain)
  }
  pub fn full_name(&self) -> String {
    format!("@{}@{}", &self.name, &self.domain)
  }
}

pub fn scrape_text_for_mentions(text: &str) -> Vec<MentionData> {
  let mut out: Vec<MentionData> = Vec::new();
  for caps in MENTIONS_REGEX.captures_iter(text) {
    out.push(MentionData {
      name: caps["name"].to_string(),
      domain: caps["domain"].to_string(),
    });
  }
  out.into_iter().unique().collect()
}

pub fn is_valid_username(name: &str) -> bool {
  VALID_USERNAME_REGEX.is_match(name)
}

pub fn is_valid_community_name(name: &str) -> bool {
  VALID_COMMUNITY_NAME_REGEX.is_match(name)
}

pub fn is_valid_post_title(title: &str) -> bool {
  VALID_POST_TITLE_REGEX.is_match(title)
}

#[cfg(test)]
mod tests {
  use crate::{
    is_valid_community_name,
    is_valid_post_title,
    is_valid_username,
    remove_slurs,
    scrape_text_for_mentions,
    slur_check,
    slurs_vec_to_str,
  };

  #[test]
  fn test_mentions_regex() {
    let text = "Just read a great blog post by [@tedu@honk.teduangst.com](/u/test). And another by !test_community@fish.teduangst.com . Another [@lemmy@lemmy-alpha:8540](/u/fish)";
    let mentions = scrape_text_for_mentions(text);

    assert_eq!(mentions[0].name, "tedu".to_string());
    assert_eq!(mentions[0].domain, "honk.teduangst.com".to_string());
    assert_eq!(mentions[1].domain, "lemmy-alpha:8540".to_string());
  }

  #[test]
  fn test_valid_register_username() {
    assert!(is_valid_username("Hello_98"));
    assert!(is_valid_username("ten"));
    assert!(!is_valid_username("Hello-98"));
    assert!(!is_valid_username("a"));
    assert!(!is_valid_username(""));
  }

  #[test]
  fn test_valid_community_name() {
    assert!(is_valid_community_name("example"));
    assert!(is_valid_community_name("example_community"));
    assert!(!is_valid_community_name("Example"));
    assert!(!is_valid_community_name("Ex"));
    assert!(!is_valid_community_name(""));
  }

  #[test]
  fn test_valid_post_title() {
    assert!(is_valid_post_title("Post Title"));
    assert!(is_valid_post_title("   POST TITLE 😃😃😃😃😃"));
    assert!(!is_valid_post_title("\n \n \n \n    		")); // tabs/spaces/newlines
  }

  #[test]
  fn test_slur_filter() {
    let test =
      "coons test dindu ladyboy tranny retardeds. Capitalized Niggerz. This is a bunch of other safe text.";
    let slur_free = "No slurs here";
    assert_eq!(
      remove_slurs(&test),
      "*removed* test *removed* *removed* *removed* *removed*. Capitalized *removed*. This is a bunch of other safe text."
        .to_string()
    );

    let has_slurs_vec = vec![
      "Niggerz",
      "coons",
      "dindu",
      "ladyboy",
      "retardeds",
      "tranny",
    ];
    let has_slurs_err_str = "No slurs - Niggerz, coons, dindu, ladyboy, retardeds, tranny";

    assert_eq!(slur_check(test), Err(has_slurs_vec));
    assert_eq!(slur_check(slur_free), Ok(()));
    if let Err(slur_vec) = slur_check(test) {
      assert_eq!(&slurs_vec_to_str(slur_vec), has_slurs_err_str);
    }
  }

  // These helped with testing
  // #[test]
  // fn test_send_email() {
  //  let result =  send_email("not a subject", "test_email@gmail.com", "ur user", "<h1>HI there</h1>");
  //   assert!(result.is_ok());
  // }
}

lazy_static! {
  static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$").unwrap();
  static ref PII_REGEX: Regex = Regex::new(r"(\+\d{1,2}\s)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}").unwrap();
  static ref MD_IMAGE_REGEX: Regex = Regex::new(r"!\[[\s\S]*?\]\([\s\S]*?\)").unwrap();
  static ref SLUR_REGEX: Regex = RegexBuilder::new(r"([^\p{P}\s]*?(f(a|4)g(got|g)?){1,}|maricos?|(n(i|1)gg((a|er)?(s|z)?)){1,}|(nig){2,}|dindu(s?){1,}|mudslime?s?|(k(i|y)kes?){1,}|(mongoloids?){1,}|(towel\s*heads?){1,}|\bspi(c|k)s?\b|(spi(c|k)s){2,}|\bchinks?|(chinks?){1,}|(niglets?){1,}|be(a|@|4)ners?|\bjaps?\b|(japs){2,}|\bcoons?\b|(coons?){2,}|(jungle\s*bunn(y|ies?)){1,}|(jigg?aboo?s?){1,}|\bpakis?\b|(pakis?){2,}|(rag\s*heads?){1,}|(gooks?){1,}|(cuntboy?){1,}|(feminazis?){1,}|(whor(es?|ing)){1,}|\bslut(s|t?y)?|(slut(s|t?y)){2,}|\btr(a|@)nn?(y|ies?)|(tr(a|@)nn?(y|ies?)){1,}|(l(a|@|4)dyboy(s?)){1,}|ret(a|4)rd?(s|ed)?|(hymie){1,}|(porch\s?monkey){1,}|(zh(y|i)d(ovka)?){1,}|\bching\s?chong\b|(ching\s?chong\s?){1,}|(chong\s?ching\s?){1,}|(hefem(a|@|4)le){1,}|(dickgirl){1,}|(hermie){1,}|(\babb?o\b)|(abb?o){2,}|(boong){1,})").case_insensitive(true).build().unwrap();
  static ref USERNAME_MATCHES_REGEX: Regex = Regex::new(r"/u/[a-zA-Z][0-9a-zA-Z_]*").unwrap();
  // TODO keep this old one, it didn't work with port well tho
  // static ref MENTIONS_REGEX: Regex = Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)").unwrap();
  static ref MENTIONS_REGEX: Regex = Regex::new(r"@(?P<name>[\w.]+)@(?P<domain>[a-zA-Z0-9._:-]+)").unwrap();
  static ref VALID_USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]{3,20}$").unwrap();
  static ref VALID_COMMUNITY_NAME_REGEX: Regex = Regex::new(r"^[a-z0-9_]{3,20}$").unwrap();
  static ref VALID_POST_TITLE_REGEX: Regex = Regex::new(r".*\S.*").unwrap();
  pub static ref WEBFINGER_COMMUNITY_REGEX: Regex = Regex::new(&format!(
    "^group:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  pub static ref WEBFINGER_USER_REGEX: Regex = Regex::new(&format!(
    "^acct:([a-z0-9_]{{3, 20}})@{}$",
    Settings::get().hostname
  ))
  .unwrap();
  pub static ref CACHE_CONTROL_REGEX: Regex =
    Regex::new("^((text|image)/.+|application/javascript)$").unwrap();
}

pub struct Keypair {
  pub private_key: String,
  pub public_key: String,
}

/// Generate the asymmetric keypair for ActivityPub HTTP signatures.
pub fn generate_actor_keypair() -> Result<Keypair, Error> {
  let rsa = Rsa::generate(2048)?;
  let pkey = PKey::from_rsa(rsa)?;
  let public_key = pkey.public_key_to_pem()?;
  let private_key = pkey.private_key_to_pem_pkcs8()?;
  let key_to_string = |key| match String::from_utf8(key) {
    Ok(s) => Ok(s),
    Err(e) => Err(Error::new(
      ErrorKind::Other,
      format!("Failed converting key to string: {}", e),
    )),
  };
  Ok(Keypair {
    private_key: key_to_string(private_key)?,
    public_key: key_to_string(public_key)?,
  })
}

pub enum EndpointType {
  Community,
  User,
  Post,
  Comment,
  PrivateMessage,
}

pub fn get_apub_protocol_string() -> &'static str {
  if Settings::get().federation.tls_enabled {
    "https"
  } else {
    "http"
  }
}

/// Generates the ActivityPub ID for a given object type and ID.
pub fn make_apub_endpoint(endpoint_type: EndpointType, name: &str) -> Url {
  let point = match endpoint_type {
    EndpointType::Community => "c",
    EndpointType::User => "u",
    EndpointType::Post => "post",
    EndpointType::Comment => "comment",
    EndpointType::PrivateMessage => "private_message",
  };

  Url::parse(&format!(
    "{}://{}/{}/{}",
    get_apub_protocol_string(),
    Settings::get().hostname,
    point,
    name
  ))
  .unwrap()
}
