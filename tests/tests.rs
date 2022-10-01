use engine_rs::binance_websocket::{StreamTicker, TickerMessage};
use engine_rs::risk::Drawdown;
use engine_rs::util::Double;

use actix::{Actor, Context, Handler, Message};

use dotenv::dotenv;

#[actix_rt::test]
async fn test_drawdown() {
  let addr1 = Drawdown::new(vec![]).start();

  let res = addr1.send(Double(0.)).await.unwrap();

  assert_eq!(res, 0.);

  let res = addr1.send(Double(1.)).await.unwrap();

  assert_eq!(res, 0.);

  let res = addr1.send(Double(0.5)).await.unwrap();

  assert_eq!(res, 0.5);

  let res = addr1.send(Double(0.)).await.unwrap();

  assert_eq!(res, 1.);
}

#[actix_rt::test]
async fn test_ticker() {
  env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
  dotenv().ok();

  let rsa = ReceivedSomethingActor::new().start();

  let st = StreamTicker::new(vec![rsa.clone().recipient()]);

  actix::spawn(st.run("btcusdt"));

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let rs = rsa.send(ReceivedSomethingMessage);

    if rs.await.unwrap() {
      break;
    }
  }
}

#[derive(Message)]
#[rtype(result = "bool")]
struct ReceivedSomethingMessage;

struct ReceivedSomethingActor {
  rcvd: bool,
}

impl ReceivedSomethingActor {
  fn new() -> Self {
    Self { rcvd: false }
  }
}

impl Actor for ReceivedSomethingActor {
  type Context = Context<Self>;
}

impl Handler<TickerMessage> for ReceivedSomethingActor {
  type Result = f64;

  fn handle(
    &mut self,
    msg: TickerMessage,
    _ctx: &mut Context<Self>,
  ) -> Self::Result {
    log::info!("Ticker msg received: {:?}", msg);
    self.rcvd = true;
    0.
  }
}

impl Handler<ReceivedSomethingMessage> for ReceivedSomethingActor {
  type Result = bool;

  fn handle(
    &mut self,
    _: ReceivedSomethingMessage,
    _: &mut Context<Self>,
  ) -> Self::Result {
    self.rcvd
  }
}
