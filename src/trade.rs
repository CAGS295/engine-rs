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
use chrono::{DateTime, Utc};
use std::sync::mpsc::channel;

use crate::policy_maker::PolicyDecision;

pub struct TradeActor {
  pub arbiter: Arbiter,
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

#[derive(Message, Clone, Debug)]
#[rtype(result = "Result<Transaction, binance::errors::Error>")]
pub struct Buy {
  pub symbol: String,
  pub quantity: f64,
  pub price: f64,
  pub timestamp: DateTime<Utc>,
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "Result<Transaction, binance::errors::Error>")]
pub struct Sell {
  pub symbol: String,
  pub quantity: f64,
  pub price: f64,
  pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Hold {
  pub symbol: String,
  pub timestamp: DateTime<Utc>,
}

impl Handler<PolicyDecision> for TradeActor {
  type Result = ();

  fn handle(&mut self, msg: PolicyDecision, _ctx: &mut Context<Self>) {
    match msg {
      PolicyDecision::BuyAction(buy) => self
        .buy(buy)
        .map_err(|e| log::warn!("Error buying: {:?}", e))
        .ok(),
      PolicyDecision::SellAction(sell) => self
        .sell(sell)
        .map_err(|e| log::warn!("Error selling: {:?}", e))
        .ok(),
      PolicyDecision::HoldAction(hold) => {
        println!("Hold: {:?}", hold);
        None
      }
    };
  }
}

impl Handler<Buy> for TradeActor {
  type Result = Result<Transaction, binance::errors::Error>;

  fn handle(&mut self, msg: Buy, _ctx: &mut Context<Self>) -> Self::Result {
    self.buy(msg)
  }
}

impl Default for TradeActor {
  fn default() -> Self {
    Self {
      arbiter: Arbiter::new(),
    }
  }
}

impl TradeActor {
  pub fn new() -> Self {
    Self {
      arbiter: Arbiter::new(),
    }
  }

  fn buy(&mut self, msg: Buy) -> Result<Transaction, binance::errors::Error> {
    log::info!("ORDER: {:?}", msg);
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
    self.sell(msg)
  }
}

impl TradeActor {
  fn sell(&mut self, msg: Sell) -> Result<Transaction, binance::errors::Error> {
    log::info!("ORDER: {:?}", msg);
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
    time_in_force: Some(TimeInForce::FOK),
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
  let market_sell = OrderRequest {
    symbol: symbol.to_string(),
    quantity: Some(quantity),
    price: Some(price),
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::FOK),
    side: OrderSide::Sell,
    ..OrderRequest::default()
  };

  account.place_order(market_sell).await
}

#[cfg(test)]
mod test {
  use super::*;
  use dotenv::dotenv;

  #[actix_rt::test]
  async fn test_actor_sell() {
    dotenv().ok();
    let trade_actor = TradeActor::new().start();
    let res = trade_actor
      .send(Sell {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.001,
        price: 10000.0,
        timestamp: Utc::now(),
      })
      .await;
    println!("{:?}", res);
    assert!(res.is_ok());
  }

  #[actix_rt::test]
  async fn test_actor_buy() {
    dotenv().ok();
    let trade_actor = TradeActor::new().start();
    let res = trade_actor
      .send(Buy {
        symbol: "BTCUSDT".to_string(),
        quantity: 0.001,
        price: 10000.0,
        timestamp: Utc::now(),
      })
      .await;
    assert!(res.is_ok());
  }
}
