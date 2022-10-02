use crate::actors::mid_price::MidPrice;

use crate::actors::mid_price::MidPriceResponse;
use crate::trade::{Buy, Hold, Sell};
use crate::util::{deserialize_from_str, MovingAverageMessage};
use actix::{Actor, Context, Handler, Message, MessageResult, Recipient};
use chrono::Utc;
use serde::Deserialize;

pub struct PolicyMaker {
  current_true_price: f64,
  prev_true_price: f64,
  frame: PolicyFrame,
  recipients: Vec<Recipient<PolicyDecision>>, // TODO
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

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum PolicyDecision {
  BuyAction(Buy),
  SellAction(Sell),
  HoldAction(Hold),
}

impl PolicyMaker {
  fn new(recipients: Vec<Recipient<PolicyDecision>>) -> Self {
    Self {
      current_true_price: 0.0,
      prev_true_price: 0.0,
      frame: PolicyFrame {
        symbol: "".to_string(),
        moving_average_gradient: 0.0,
        true_price_gradient: 0.0,
        moving_average_price: 0.0,
        true_price: 0.0,
        prev_decision: None,
      },
      recipients,
    }
  }
  async fn run(self) {}

  fn propagate_decision(&self, decision: PolicyDecision) {
    for recipient in self.recipients.iter() {
      recipient.do_send(decision.clone());
    }
  }

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

impl Handler<MidPrice> for PolicyMaker {
  type Result = MessageResult<MidPrice>;

  // Handle true price (TickerMessage), always keep the latest true price
  // The actual decision making is done when handling moving average message
  fn handle(
    &mut self,
    msg: MidPrice,
    _ctx: &mut Context<Self>,
  ) -> Self::Result {
    let prev_true_price = self.current_true_price;
    self.current_true_price = msg.price;

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

    MessageResult(MidPriceResponse::MovingAverage(0.))
  }
}

impl Handler<MovingAverageMessage> for PolicyMaker {
  type Result = f64;
  // Handle moving average message. Receive message then make a policy decision
  fn handle(
    &mut self,
    msg: MovingAverageMessage,
    _ctx: &mut Context<Self>,
  ) -> f64 {
    let _prev_moving_price = self.frame.moving_average_price;

    self.frame.moving_average_gradient =
      msg.0 - self.frame.moving_average_price;
    self.frame.moving_average_price = msg.0;

    let decision = self.make_policy_decision(&self.frame);
    log::error!("Decision: {:?}", decision);
    self.propagate_decision(decision);
    msg.0
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

#[cfg(test)]
mod test {
  use crate::actors::mid_price::MidPrice;
  use crate::util::MovingAverageMessage;

  use super::PolicyDecision;
  use super::PolicyFrame;
  use super::PolicyMaker;

  use super::{Buy, Sell};
  use crate::trade::TradeActor;
  use actix::Actor;
  use actix::Arbiter;
  use chrono::Utc;

  #[actix_rt::test]
  #[ignore]
  async fn test_policy_buy() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let arbiter = Arbiter::new();
    let trade_actor = TradeActor { arbiter }.start().recipient();
    let sut = PolicyMaker::new(vec![trade_actor]);
    let addr = sut.start();
    addr.do_send(MidPrice {
      symbol: "BTCUSDT".to_string(),
      price: 100.,
    });
    addr.send(MovingAverageMessage(10.)).await.unwrap();

    addr
      .send(MidPrice {
        symbol: "BTCUSDT".to_string(),
        price: 1000.,
      })
      .await
      .unwrap();
    addr.do_send(MovingAverageMessage(100.));
    loop {
      //println!("waiting");
    }
  }

  #[test]
  fn test_should_buy() {
    let sell = Sell {
      symbol: "btcusdt".to_string(),
      quantity: 0.1,
      price: 10.0,
      timestamp: Utc::now(),
    };

    let frame = PolicyFrame {
      symbol: "btcusdt".to_string(),
      moving_average_gradient: 1.0,
      true_price_gradient: 1.0,
      moving_average_price: 10.0,
      true_price: 20.0,
      prev_decision: Some(PolicyDecision::SellAction(sell)),
    };

    let result = super::should_buy(&frame);
    assert_eq!(
      result, true,
      "True price trending upwards and higher than avg price, should buy"
    );
  }

  #[test]
  fn test_not_should_buy_again() {
    let buy = Buy {
      symbol: "btcusdt".to_string(),
      quantity: 0.1,
      price: 10.0,
      timestamp: Utc::now(),
    };

    let frame = PolicyFrame {
      symbol: "btcusdt".to_string(),
      moving_average_gradient: 1.0,
      true_price_gradient: 1.0,
      moving_average_price: 10.0,
      true_price: 20.0,
      prev_decision: Some(PolicyDecision::BuyAction(buy)),
    };

    let result = super::should_buy(&frame);
    assert_eq!(
      result, false,
      "Prev decision was already buy, should not buy again"
    );
  }
}
