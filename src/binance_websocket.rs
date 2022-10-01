use crate::util::deserialize_from_str;
use actix::{Actor, Context, Handler, Message, Recipient};
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

use dotenv::dotenv;
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

struct StreamTicker {
  recipients: Vec<Recipient<TickerMessage>>,
}

#[derive(Message)]
#[rtype(result = "()")]
#[derive(Deserialize, Debug, Clone)]
struct TickerMessage {
  u: u64,
  s: String,
  #[serde(deserialize_with = "deserialize_from_str")]
  b: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  B: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  a: f64,
  #[serde(deserialize_with = "deserialize_from_str")]
  A: f64,
}

impl StreamTicker {
  async fn run(self, symbol: &str) {
    let result = open_partial_depth_stream(symbol).await;
    let (_res, mut ws) = result.unwrap();
    while let Some(msg) = ws.next().await {
      match msg {
        Ok(ws::Frame::Text(txt)) => {
          let v: TickerMessage = serde_json::from_slice(&txt).unwrap();
          log::info!("Server: {v:?}");
          self.recipients.iter().for_each(|r| r.do_send(v.clone()));
        }
        _ => {}
      }
    }
  }
}

struct DActor {
  rcvd: bool,
}
impl Actor for DActor {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Context<Self>) {
    log::info!("Actor is alive");
  }

  fn stopped(&mut self, _ctx: &mut Context<Self>) {
    println!("Actor is stopped");
  }
}

impl Handler<TickerMessage> for DActor {
  type Result = ();
  fn handle(&mut self, msg: TickerMessage, _ctx: &mut Context<Self>) {
    log::error!("Ticker msg received: {:?}", msg);
    self.rcvd = true;
  }
}
#[cfg(test)]
mod test {
  use super::*;
  use dotenv::dotenv;
  #[actix_rt::test]
  async fn test_open_partial_depth_stream() {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let dummy_actor = DActor { rcvd: false };
    let act_rcv = dummy_actor.start().recipient();
    let sut = StreamTicker {
      recipients: vec![act_rcv],
    };
    actix::spawn(sut.run("btcusdt")).await.unwrap();
    loop {}
  }
}
