use crate::util::{deserialize_from_str, tls_web_client};
use actix::{Message, Recipient};
use actix_codec::Framed;
use awc::error::WsClientError;
use awc::ws;
use awc::ws::Codec;
use awc::BoxedSocket;
use awc::Client;
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json;

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
  #[serde(alias = "u")]
  pub update_id: u64,
  #[serde(alias = "s")]
  pub symbol: String,
  #[serde(deserialize_with = "deserialize_from_str", alias = "b")]
  pub best_bid_price: f64,
  #[serde(deserialize_with = "deserialize_from_str", alias = "B")]
  pub best_bid_qty: f64,
  #[serde(deserialize_with = "deserialize_from_str", alias = "a")]
  pub best_ask_price: f64,
  #[serde(deserialize_with = "deserialize_from_str", alias = "A")]
  pub best_ask_qty: f64,
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
  client: Client,
  book_ticker_recipients: Vec<Recipient<TickerMessage>>,
  user_data_account_update_recipients: Vec<Recipient<AccountUpdateMessage>>,
}

impl BinanceIngestor {
  pub fn new(
    book_ticker_recipients: Vec<Recipient<TickerMessage>>,
    user_data_account_update_recipients: Vec<Recipient<AccountUpdateMessage>>,
  ) -> Self {
    Self {
      client: tls_web_client(),
      book_ticker_recipients,
      user_data_account_update_recipients,
    }
  }

  async fn get_stream(
    &self,
  ) -> Result<Framed<BoxedSocket, Codec>, WsClientError> {
    //let user_stream: UserStream = Binance::new_with_env(&Config::testnet());

    //let listen_key = user_stream
    //  .start()
    //  .await
    //  .map_err(|_| WsClientError::MissingConnectionHeader)?
    //  .listen_key;

    self
      .client
      .ws(
        "wss://testnet.binance.vision/stream?streams=btcusdt@bookTicker"
          .to_string(),
      )
      .connect()
      .await
      .map(|x| x.1)
  }

  pub async fn run(self) {
    let mut ws = self.get_stream().await.unwrap();

    while let Some(msg) = ws.next().await {
      if let Ok(ws::Frame::Text(txt)) = msg {
        match serde_json::from_slice::<BinanceMessage>(&txt) {
          Ok(v) => match v.data {
            BinanceMessageContent::BookTicker(tm) => {
              log::debug!("Received ticker message: {tm:?}");

              for r in &self.book_ticker_recipients {
                r.do_send(tm.clone());
              }
            }
            BinanceMessageContent::UserDataAccountUpdate(aum) => {
              log::debug!("Received account update message: {aum:?}");

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
