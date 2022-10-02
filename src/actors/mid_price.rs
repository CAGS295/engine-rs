use actix::MessageResult;

use crate::{Actor, Context, Handler, Message, Recipient};

#[derive(Message)]
#[rtype(result = "MidPriceResponse")]
pub struct MidPrice(pub f64);

#[derive(Debug)]
pub enum MidPriceResponse {
  MovingAverage(f64),
  Policy(f64),
}

use crate::binance_websocket::TickerMessage;

pub struct MidPriceActor {
  pub subscribers: Vec<Recipient<MidPrice>>,
}

impl MidPriceActor {
  pub fn new(subscribers: Vec<Recipient<MidPrice>>) -> Self {
    Self { subscribers }
  }
}

impl Actor for MidPriceActor {
  type Context = Context<Self>;
}

impl Handler<TickerMessage> for MidPriceActor {
  type Result = MessageResult<TickerMessage>;

  fn handle(
    &mut self,
    msg: TickerMessage,
    _ctx: &mut Self::Context,
  ) -> Self::Result {
    let TickerMessage { b, a, .. } = msg;
    let price = (b + a) / 2f64;
    for consumer in &self.subscribers {
      consumer.do_send(MidPrice(price));
    }
    MessageResult(price)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[actix_rt::test]
  async fn positive() {
    let actor = MidPriceActor::new(vec![]);
    let addr = actor.start();
    let res = addr
      .send(TickerMessage {
        b: 1.0,
        a: 1.5,
        ..Default::default()
      })
      .await
      .unwrap();
    assert_eq!(res, 1.25);
  }
}
