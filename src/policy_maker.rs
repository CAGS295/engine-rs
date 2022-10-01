use crate::binance_websocket::TickerMessage;
use crate::trade::{Buy, Hold, Sell};
use crate::util::deserialize_from_str;
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
    if should_buy(frame) {
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
    }
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

impl Handler<TickerMessage> for PolicyMaker {
  type Result = ();
  // Handle true price (TickerMessage), always keep the latest true price
  // The actual decision making is done when handling moving average message
  fn handle(&mut self, msg: TickerMessage, _ctx: &mut Context<Self>) {
    log::error!("Ticker msg received: {:?}", msg);
    let prev_true_price = self.current_true_price;
    self.current_true_price = msg.best_ask_price;

    let frame = PolicyFrame {
      true_price_gradient: self.current_true_price - prev_true_price,
      true_price: self.current_true_price,
      symbol: msg.symbol,
      prev_decision: self.frame.prev_decision.take(),
      // TODO: update
      moving_average_gradient: 0.,
      moving_average_price: 0.,
    };
    self.frame = frame;
  }
}

impl Handler<MovingMessage> for PolicyMaker {
  type Result = ();
  // Handle moving average message. Receive message then make a policy decision
  fn handle(&mut self, msg: MovingMessage, _ctx: &mut Context<Self>) {
    let _prev_moving_price = self.frame.moving_average_price;

    self.frame.moving_average_gradient =
      msg.best_ask_price - self.frame.moving_average_price; // TODO MovingAvgMsg
    self.frame.moving_average_price = msg.best_ask_price; //

    self.make_policy_decision(&self.frame);
  }
}

fn should_buy(frame: &PolicyFrame) -> bool {
  is_rising_trend(frame)
    && frame.moving_average_price < frame.true_price
    && !(matches!(frame.prev_decision, Some(PolicyDecision::BuyAction(_))))
}

fn should_sell(frame: &PolicyFrame) -> bool {
  is_downward_trend(frame)
    && frame.moving_average_price > frame.true_price
    && !(matches!(frame.prev_decision, Some(PolicyDecision::SellAction(_))))
}

fn is_rising_trend(frame: &PolicyFrame) -> bool {
  frame.moving_average_gradient > 0.0 && frame.true_price_gradient > 0.0
}

fn is_downward_trend(frame: &PolicyFrame) -> bool {
  frame.moving_average_gradient < 0.0 && frame.true_price_gradient < 0.0
}
