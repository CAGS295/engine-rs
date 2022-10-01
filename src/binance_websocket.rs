use actix_codec::Framed;
use awc::error::WsClientError;
use awc::ws::Codec;
use awc::{BoxedSocket, ClientResponse};
use openssl::ssl::{SslConnector, SslMethod};

fn build_client() -> awc::Client {
  let ssl_connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
  let awc_connector = awc::Connector::new().openssl(ssl_connector);
  awc::Client::builder().connector(awc_connector).finish()
}

// example symbol = btcusdt
pub async fn open_partial_depth_stream(
  symbol: &str,
) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
  let client = build_client();

  client
    .ws(format!(
      "wss://stream.binance.com:9443/ws/{}@depth5",
      symbol
    ))
    .connect()
    .await
}
