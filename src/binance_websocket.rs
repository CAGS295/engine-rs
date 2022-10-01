use crate::actors::mid_price::MidPrice;
use crate::util::deserialize_from_str;
use actix::MessageResult;
use actix::{Message, Recipient};
use actix_codec::Framed;
use awc::error::WsClientError;
use awc::ws;
use awc::ws::Codec;
use awc::{BoxedSocket, ClientResponse};
use binance::api::Binance;
use binance::config::Config;
use binance::rest_model::UserDataStream;
use binance::userstream::UserStream;
use futures_util::StreamExt;

use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;
use serde_json;

fn build_client() -> awc::Client {
  let ssl = SslConnector::builder(SslMethod::tls()).unwrap().build();
  let conn = awc::Connector::new().openssl(ssl);
  awc::Client::builder().connector(conn).finish()
}

// https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-book-ticker-streams
// example symbol = btcusdt
pub async fn open_book_ticket_stream(
  symbol: &str,
) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
  let client = build_client();

  client
    //.ws(format!("wss://stream.binance.com:9443/ws/{symbol:}@depth5"))
    .ws(format!(
      "wss://testnet.binance.vision/ws/{symbol:}@bookTicker"
    ))
    .connect()
    .await
}

// https://binance-docs.github.io/apidocs/spot/en/#user-data-streams
// Calls Binance REST API endpoint to get a listenKey, then use the listenKey to open a WebSocket stream
// In theory, this will take care of keeping the stream alive by sending back Ping/Keep-alive requests
pub async fn open_user_data_stream(
) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
  dotenv().ok();
  let user_stream: UserStream = Binance::new_with_env(&Config::testnet());
  let answer: UserDataStream = user_stream
    .start()
    .await
    .map_err(|_| WsClientError::MissingConnectionHeader)?;
  let listen_key: String = answer.listen_key;
  println!("Listen key: {:?}", listen_key);
  let client = build_client();

  client
    //.ws(format!("wss://stream.binance.com:9443/ws/{listen_key:}"))
    .ws(format!("wss://testnet.binance.vision/ws/{listen_key:}"))
    .connect()
    .await
}

#[allow(non_snake_case)]
#[derive(Message)]
#[rtype(result = "f64")]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct TickerMessage {
  #[serde(deserialize_with = "deserialize_from_str", rename = "u")]
  pub update_id: u64,

  #[serde(deserialize_with = "deserialize_from_str", rename = "s")]
  pub symbol: String,

  #[serde(deserialize_with = "deserialize_from_str", rename = "b")]
  pub best_bid_price: f64,

  #[serde(deserialize_with = "deserialize_from_str", rename = "B")]
  pub best_bid_qty: f64,

  #[serde(deserialize_with = "deserialize_from_str", rename = "a")]
  pub best_ask_price: f64,

  #[serde(deserialize_with = "deserialize_from_str", rename = "A")]
  pub best_ask_qty: f64,
}
#[derive(Default)]
pub struct StreamTicker {
  recipients: Vec<Recipient<TickerMessage>>,
}

impl StreamTicker {
  pub fn new(recipients: Vec<Recipient<TickerMessage>>) -> Self {
    Self { recipients }
  }

  pub async fn run(self, symbol: &str) {
    let result = open_book_ticket_stream(symbol).await;

    let (_, mut ws) = result.unwrap();

    while let Some(msg) = ws.next().await {
      if let Ok(ws::Frame::Text(txt)) = msg {
        match serde_json::from_slice::<TickerMessage>(&txt) {
          Ok(v) => {
            log::info!("Stream ticker received: {v:?}");

            for r in &self.recipients {
              r.do_send(v.clone());
            }
          }
          Err(e) => {
            log::error!("Stream ticker couldn't deserialize message: {txt:?}. Error: {e:?}");
          }
        }
      }
    }
  }
}
