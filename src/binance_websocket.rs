use crate::util::deserialize_from_str;
use actix::{Message, Recipient};
use actix_codec::Framed;

use awc::error::WsClientError;
use awc::ws;
use awc::ws::Codec;
use awc::{BoxedSocket, ClientResponse};
use futures_util::StreamExt;

use binance::api::Binance;
use binance::config::Config;
use binance::rest_model::UserDataStream;
use binance::userstream::UserStream;

use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use serde_json;

fn build_client() -> awc::Client {
  let ssl = SslConnector::builder(SslMethod::tls()).unwrap().build();
  let conn = awc::Connector::new().openssl(ssl);
  awc::Client::builder().connector(conn).finish()
}

// https://binance-docs.github.io/apidocs/spot/en/#partial-book-depth-streams
// example symbol = btcusdt
pub async fn open_partial_depth_stream(
  symbol: &str,
) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
  let user_stream: UserStream = Binance::new_with_env(&Config::testnet());

  let answer: UserDataStream = user_stream
    .start()
    .await
    .map_err(|_| WsClientError::MissingConnectionHeader)?;

  let listen_key: String = answer.listen_key;

  let client = build_client();

  client
    .ws(format!(
      "wss://testnet.binance.vision/stream?streams={symbol:}@bookTicker/{listen_key:}"
    ))
    .connect()
    .await
}

// https://binance-docs.github.io/apidocs/spot/en/#user-data-streams
// Calls Binance REST API endpoint to get a listenKey, then use the listenKey to open a WebSocket stream
// In theory, this will take care of keeping the stream alive by sending back Ping/Keep-alive requests
pub async fn open_user_data_stream(
) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
  let user_stream: UserStream = Binance::new_with_env(&Config::testnet());

  let answer: UserDataStream = user_stream
    .start()
    .await
    .map_err(|_| WsClientError::MissingConnectionHeader)?;

  let listen_key: String = answer.listen_key;

  let client = build_client();

  client
    .ws(format!("wss://testnet.binance.vision/ws/{listen_key:}"))
    .connect()
    .await
}

#[derive(Deserialize)]
struct BinanceMessage {
  data: BinanceMessageContent,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BinanceMessageContent {
  BookTicker(TickerMessage),
  UserDataAccountUpdate(AccountUpdateMessage),
}

#[allow(non_snake_case)]
#[derive(Message, Deserialize, Debug, Clone, Default)]
#[rtype(result = "f64")]
pub struct TickerMessage {
  pub u: u64,
  pub s: String,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub b: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub B: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub a: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub A: f64,
}

#[allow(non_snake_case)]
#[derive(Message, Deserialize, Debug, Clone, Default)]
#[rtype(result = "()")]
pub struct AccountUpdateMessage {
  pub e: String,
  pub E: u64,
  pub u: u64,
  pub B: Vec<Balance>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Balance {
  pub a: String,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub f: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  pub l: f64,
}

#[derive(Default)]
pub struct BinanceIngestor {
  book_ticker_recipients: Vec<Recipient<TickerMessage>>,
  user_data_account_update_recipients: Vec<Recipient<AccountUpdateMessage>>,
}

impl BinanceIngestor {
  pub fn new(
    book_ticker_recipients: Vec<Recipient<TickerMessage>>,
    user_data_account_update_recipients: Vec<Recipient<AccountUpdateMessage>>,
  ) -> Self {
    Self {
      book_ticker_recipients,
      user_data_account_update_recipients,
    }
  }

  pub async fn run(self, symbol: &str) {
    let result = open_partial_depth_stream(symbol).await;

    let (_, mut ws) = result.unwrap();

    while let Some(msg) = ws.next().await {
      if let Ok(ws::Frame::Text(txt)) = msg {
        match serde_json::from_slice::<BinanceMessage>(&txt) {
          Ok(v) => match v.data {
            BinanceMessageContent::BookTicker(tm) => {
              log::info!("Received ticker message: {tm:?}");

              for r in &self.book_ticker_recipients {
                r.do_send(tm.clone());
              }
            }
            BinanceMessageContent::UserDataAccountUpdate(aum) => {
              log::info!("Received account update message: {aum:?}");

              for r in &self.user_data_account_update_recipients {
                r.do_send(aum.clone());
              }
            }
          },
          Err(e) => {
            log::error!("Binance ingestor couldn't deserialize message: {txt:?}. Error: {e:?}");
          }
        }
      }
    }
  }
}
