use actix::Message;
use serde::{de::Error, Deserialize};

use awc::Client;

use openssl::ssl::{SslConnector, SslMethod};

#[derive(Message)]
#[rtype(result = "f64")]
pub struct Double(pub f64);

#[derive(Message)]
#[rtype(result = "f64")]
pub struct Return(pub f64);

#[derive(Message)]
#[rtype(result = "f64")]
pub struct MovingAverageMessage(pub f64);

#[derive(Message)]
#[rtype(result = "Option<f64>")]
pub struct SharpeRatio(pub f64);

pub fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
  D: serde::de::Deserializer<'de>,
  T: std::str::FromStr,
{
  let s = String::deserialize(deserializer)?;

  s.parse().map_err(|_| {
    Error::custom("failed to parse deserialized value to desired type")
  })
}

pub(crate) fn tls_web_client() -> Client {
  let ssl = SslConnector::builder(SslMethod::tls()).unwrap().build();
  let conn = awc::Connector::new().openssl(ssl);
  awc::Client::builder().connector(conn).finish()
}
