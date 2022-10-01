use actix_codec::{Decoder, Framed};
use actix_web::dev::Payload;
use actix_web::error::PayloadError;
use actix_web::web::Bytes;
use awc::{BoxedSocket, ClientResponse};
use awc::error::{SendRequestError, WsClientError};
use awc::ws::Codec;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Deserialize;

const BINANCE_API_KEY: &str = "ShI5Z0Ee494UQBSmutK47UoInUIYxiMBdPeZgWOq6UgTzNwIraEGj72zak0KaOT8";
const BINANCE_SECRET_KEY: &str = "kYBnxjKd3nUJxdo7OiwU8cERfHmYNN1iHE3f1A9iD9BtVJ6DRfNYQSWcbvZvgnnT";
const BINANCE_BASE_URL: &str = "https://api.binance.com";


#[derive(Deserialize)]
struct UserDataStreamBody {
    listenKey: String
}

fn build_client() -> awc::Client {
    let ssl = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let conn = awc::Connector::new().openssl(ssl);
    awc::Client::builder().connector(conn).finish()
}

// https://binance-docs.github.io/apidocs/spot/en/#partial-book-depth-streams
// example symbol = btcusdt
pub async fn open_partial_depth_stream(symbol: &str) -> Result<(ClientResponse, Framed<BoxedSocket, Codec>), WsClientError> {
    let client = build_client();
    let result = client
        .ws(format!("wss://stream.binance.com:9443/ws/{symbol:}@depth5"))
        .connect()
        .await;
    result
}

// https://binance-docs.github.io/apidocs/spot/en/#user-data-streams
// Calls Binance REST API endpoint to get a listenKey, then use the listenKey to open a WebSocket stream
// In theory, this will take care of keeping the stream alive by sending back Ping/Keep-alive requests
pub async fn open_user_data_stream() -> Result<Bytes, PayloadError> {
    let url = BINANCE_BASE_URL.to_owned() + "/api/v3/userDataStream";
    let client = build_client();
    let request = client.post(url).insert_header(("X-MBX-APIKEY", BINANCE_API_KEY));
    let mut response = request.send().await.map_err(|_| PayloadError::UnknownLength)?;
    let data = response.body().await?;
    println!("Response: {:?}", data);
    Ok(data)
}

