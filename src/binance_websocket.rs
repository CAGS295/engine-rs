use actix_codec::Framed;

use awc::error::WsClientError;
use awc::ws::Codec;
use awc::{BoxedSocket, ClientResponse};

use binance::api::Binance;
use binance::config::Config;
use binance::rest_model::UserDataStream;
use binance::userstream::UserStream;

use dotenv::dotenv;
use openssl::ssl::{SslConnector, SslMethod};

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
    .ws(format!("wss://testnet.binance.vision/ws/{symbol:}@depth5"))
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
