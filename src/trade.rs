use actix::Actor;
use actix::Arbiter;
use actix::Context;
use actix::Handler;
use actix::Message;
use binance::account::*;
use binance::api::*;
use binance::config::Config;
use binance::rest_model::Transaction;
use binance::rest_model::{OrderSide, OrderType, TimeInForce};
use std::sync::mpsc::channel;

struct TradeActor {
  arbiter: Arbiter,
}

impl Actor for TradeActor {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Context<Self>) {
    println!("Actor is alive");
  }

  fn stopped(&mut self, _ctx: &mut Context<Self>) {
    println!("Actor is stopped");
  }
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, binance::errors::Error>")]
struct Buy {
  symbol: String,
  quantity: f64,
  price: f64,
}

#[derive(Message)]
#[rtype(result = "Result<Transaction, binance::errors::Error>")]
struct Sell {
  symbol: String,
  quantity: f64,
  price: f64,
}

impl Handler<Buy> for TradeActor {
  type Result = Result<Transaction, binance::errors::Error>;

  fn handle(&mut self, msg: Buy, _ctx: &mut Context<Self>) -> Self::Result {
    let (tx, rx) = channel();

    let task = async move {
      let res = buy(msg.symbol.as_str(), msg.quantity, msg.price).await;
      tx.send(res).unwrap();
    };
    self.arbiter.spawn(task);
    rx.recv().unwrap()
  }
}

impl Handler<Sell> for TradeActor {
  type Result = Result<Transaction, binance::errors::Error>;
  fn handle(&mut self, msg: Sell, _ctx: &mut Context<Self>) -> Self::Result {
    let (tx, rx) = channel();
    let task = async move {
      let res = sell(msg.symbol.as_str(), msg.quantity, msg.price).await;
      tx.send(res).unwrap();
    };
    self.arbiter.spawn(task);
    rx.recv().unwrap()
  }
}

async fn buy(
  symbol: &str,
  quantity: f64,
  price: f64,
) -> Result<Transaction, binance::errors::Error> {
  let account: Account = Binance::new_with_env(&Config::testnet());
  let market_buy = OrderRequest {
    symbol: symbol.to_string(),
    quantity: Some(quantity),
    price: Some(price),
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::GTC),
    side: OrderSide::Buy,
    ..OrderRequest::default()
  };

  account.place_order(market_buy).await
}

async fn sell(
  symbol: &str,
  quantity: f64,
  price: f64,
) -> Result<Transaction, binance::errors::Error> {
  let account: Account = Binance::new_with_env(&Config::testnet());
  let market_buy = OrderRequest {
    symbol: symbol.to_string(),
    quantity: Some(quantity),
    price: Some(price),
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::FOK),
    side: OrderSide::Sell,
    ..OrderRequest::default()
  };

  account.place_order(market_buy).await
}

#[cfg(test)]
mod test {
  use super::*;
  use dotenv::dotenv;

  #[actix_rt::test]
  async fn test_actor_sell() {
    dotenv().ok();
    let arbiter = Arbiter::new();
    let trade_actor = TradeActor { arbiter }.start();
    let res = trade_actor
      .send(Sell {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.001,
        price: 10000.0,
      })
      .await;
    println!("{:?}", res);
    assert!(res.is_ok());
  }

  #[actix_rt::test]
  async fn test_actor_buy() {
    dotenv().ok();
    let arbiter = Arbiter::new();
    let trade_actor = TradeActor { arbiter }.start();
    let res = trade_actor
      .send(Buy {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.001,
        price: 10000.0,
      })
      .await;
    assert!(res.is_ok());
  }
}
