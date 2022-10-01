use crate::actors::mid_price::MidPrice;
use crate::binance_websocket::TickerMessage;
use crate::trade::{Buy, Hold, Sell};
use crate::util::{deserialize_from_str, MovingAverageMessage};
use actix::{Actor, Context, Handler, Message};
use chrono::Utc;
use serde::Deserialize;

pub struct PolicyMaker {
  moving_average_stream: f64,
  true_price_stream: f64,
  current_true_price: f64,
  prev_true_price: f64,
  frame: PolicyFrame,
}

impl Actor for PolicyMaker {
  type Context = Context<Self>;

  fn started(&mut self, _ctx: &mut Context<Self>) {
    println!("Actor is alive");
  }

  fn stopped(&mut self, _ctx: &mut Context<Self>) {
    println!("Actor is stopped");
  }
}

// PolicyFrame is a snapshot in time, containing all parameters
// necessary to make a policy decision
pub struct PolicyFrame {
  symbol: String,
  moving_average_gradient: f64,
  true_price_gradient: f64,
  moving_average_price: f64,
  true_price: f64,
  prev_decision: Option<PolicyDecision>,
}

enum PolicyDecision {
  BuyAction(Buy),
  SellAction(Sell),
  HoldAction(Hold),
}

impl PolicyMaker {
  fn new() -> Self {
    Self {
      moving_average_stream: 0.,
      true_price_stream: 0.,
      current_true_price: 0.,
      prev_true_price: 0.,
      frame: PolicyFrame {
        symbol: todo!(),
        moving_average_gradient: todo!(),
        true_price_gradient: todo!(),
        moving_average_price: todo!(),
        true_price: todo!(),
        prev_decision: todo!(),
      },
    }
  }
  async fn run(self) {}

  // if moving average and true price are trending upwards,
  // and moving average is below true price,
  // and last action is not buy, do buy action
  //
  // if moving average and true price are trending downwards,
  // and moving average is above true price,
  // and last action is not sell, do sell action
  //
  // else, hold and do nothing
  fn make_policy_decision(&self, frame: &PolicyFrame) -> PolicyDecision {
    return if should_buy(frame) {
      PolicyDecision::BuyAction(Buy {
        symbol: frame.symbol.clone(),
        quantity: 0.1,
        price: frame.true_price,
        timestamp: Utc::now(),
      })
    } else if should_sell(frame) {
      PolicyDecision::SellAction(Sell {
        symbol: frame.symbol.clone(),
        quantity: 0.1,
        price: frame.true_price,
        timestamp: Utc::now(),
      })
    } else {
      PolicyDecision::HoldAction(Hold {
        symbol: frame.symbol.clone(),
        timestamp: Utc::now(),
      })
    };
  }
}

#[derive(Deserialize, Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct MovingMessage {
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
impl Handler<MovingAverageMessage> for PolicyMaker {
  type Result = f64;
  // Handle moving average message. Receive message then make a policy decision
  fn handle(
    &mut self,
    msg: MovingAverageMessage,
    _ctx: &mut Context<Self>,
  ) -> f64 {
    let prev_moving_price = self.frame.moving_average_price;

    self.frame.moving_average_gradient =
      msg.0 - self.frame.moving_average_price;
    self.frame.moving_average_price = msg.0;

    self.make_policy_decision(&self.frame);
    return msg.0;
  }
}

fn should_buy(frame: &PolicyFrame) -> bool {
  is_rising_trend(&frame)
    && frame.moving_average_price < frame.true_price
    && !(matches!(frame.prev_decision, Some(PolicyDecision::BuyAction(_))))
}

fn should_sell(frame: &PolicyFrame) -> bool {
  is_downward_trend(&frame)
    && frame.moving_average_price > frame.true_price
    && !(matches!(frame.prev_decision, Some(PolicyDecision::SellAction(_))))
}

fn is_rising_trend(frame: &PolicyFrame) -> bool {
  frame.moving_average_gradient > 0.0 && frame.true_price_gradient > 0.0
}

fn is_downward_trend(frame: &PolicyFrame) -> bool {
  frame.moving_average_gradient < 0.0 && frame.true_price_gradient < 0.0
}

#[cfg(test)]
mod test {
  use super::PolicyDecision;
  use super::PolicyFrame;
  use super::PolicyMaker;

  use super::{Buy, Hold, Sell};
  use actix::Actor;
  use chrono::Utc;

  fn test_policy() {
    let sut = PolicyMaker::new();
    let addr = sut.start();
    //addr.send()
  }

  #[actix_rt::test]
  fn test_should_buy() {
    let buy = Buy {
      symbol: "btcusdt".to_string(),
      quantity: 0.1,
      price: 10.0,
      timestamp: Utc::now(),
    };

    let frame = PolicyFrame {
      symbol: "btcusdt".to_string(),
      moving_average_gradient: 2.0,
      true_price_gradient: 3.0,
      moving_average_price: 10.0,
      true_price: 10.0,
      prev_decision: Some(PolicyDecision::BuyAction(buy)),
    };
  }
}
