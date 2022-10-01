use std::{io, thread};

use actix_web::web::Bytes;
use awc::ws;
use engine_rs::binance_websocket::{
  open_partial_depth_stream, open_user_data_stream,
};
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::{select, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[actix_web::main]
async fn main() {
  env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

  log::info!("starting echo WebSocket client");

  let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
  let mut cmd_rx = UnboundedReceiverStream::new(cmd_rx);

  // run blocking terminal input reader on separate thread
  let input_thread = thread::spawn(move || loop {
    let mut cmd = String::with_capacity(32);

    if io::stdin().read_line(&mut cmd).is_err() {
      log::error!("error reading line");
      return;
    }

    cmd_tx.send(cmd).unwrap();
  });

  let user_stream = open_user_data_stream();
  let (user_res, mut user_ws) = user_stream.await.unwrap();

  let book_stream = open_partial_depth_stream("btcusdt");
  let (res, mut ws) = book_stream.await.unwrap();

  log::info!("response: {res:?}");
  log::info!("user response: {user_res:?}");
  log::info!("connected; server will echo messages sent");

  loop {
    select! {
        Some(msg) = user_ws.next() => {
            match msg {
                Ok(ws::Frame::Text(txt)) => {
                    // log echoed messages from server
                    log::info!("User Data: {txt:?}")
                }

                Ok(ws::Frame::Ping(_)) => {
                    // respond to ping probes
                    user_ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                }

                _ => {}
            }
        }

        Some(msg) = ws.next() => {
            match msg {
                Ok(ws::Frame::Text(txt)) => {
                    // log echoed messages from server
                    log::info!("Server: {txt:?}")
                }

                Ok(ws::Frame::Ping(_)) => {
                    // respond to ping probes
                    ws.send(ws::Message::Pong(Bytes::new())).await.unwrap();
                }

                _ => {}
            }
        }

        Some(cmd) = cmd_rx.next() => {
            if cmd.is_empty() {
                continue;
            }

            ws.send(ws::Message::Text(cmd.into())).await.unwrap();
        }

        else => break
    }
  }

  input_thread.join().unwrap();
}
