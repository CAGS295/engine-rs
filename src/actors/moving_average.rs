use actix::{Actor, Context, Handler, MessageResult, Recipient};

use crate::util::MovingAverageMessage;

// Moving average is 0 if number of received messages
// is not a multiple of the interval_length
pub struct MovingAverageActor {
  moving_average: f64,
  ring_buffer: Vec<f64>,
  interval_length: usize,
  subscribers: Vec<Recipient<MovingAverageMessage>>,
}

impl MovingAverageActor {
  pub fn new(
    interval_length: usize,
    subscribers: Vec<Recipient<MovingAverageMessage>>,
  ) -> Self {
    Self {
      moving_average: 0.,
      ring_buffer: vec![0.; interval_length],
      interval_length,
      subscribers,
    }
  }
}

impl Actor for MovingAverageActor {
  type Context = Context<Self>;
}

use crate::actors::mid_price::MidPrice;

use super::mid_price::MidPriceResponse;

impl Handler<MidPrice> for MovingAverageActor {
  type Result = MessageResult<MidPrice>;

  fn handle(
    &mut self,
    msg: MidPrice,
    _ctx: &mut Context<Self>,
  ) -> Self::Result {
    let empty_buffer_length =
      self.ring_buffer.iter().filter(|&n| *n == 0.).count();

    if empty_buffer_length > 1 {
      self.ring_buffer[self.interval_length - empty_buffer_length] = msg.price;

      MessageResult(MidPriceResponse::MovingAverage(0f64))
    } else if empty_buffer_length == 1 {
      self.ring_buffer[self.interval_length - empty_buffer_length] = msg.price;
      self.moving_average =
        self.ring_buffer.iter().sum::<f64>() / self.interval_length as f64;

      for s in &self.subscribers {
        s.do_send(MovingAverageMessage(self.moving_average));
      }

      MessageResult(MidPriceResponse::MovingAverage(self.moving_average))
    } else {
      self.ring_buffer.remove(0);
      self.ring_buffer.push(msg.price);
      self.moving_average =
        self.ring_buffer.iter().sum::<f64>() / self.interval_length as f64;

      for s in &self.subscribers {
        s.do_send(MovingAverageMessage(self.moving_average));
      }
      MessageResult(MidPriceResponse::MovingAverage(self.moving_average))
    }
  }
}

#[cfg(test)]
use crate::assert_matches;

#[actix_rt::test]
async fn positive() {
  let addr = MovingAverageActor::new(3, vec![]).start();
  let res = addr
    .send(MidPrice {
      price: 1.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();

  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 0.)
  });
  let res = addr
    .send(MidPrice {
      price: 2.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();
  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 0.)
  });
  let res = addr
    .send(MidPrice {
      price: 3.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();
  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 2.)
  });
  let res = addr
    .send(MidPrice {
      price: 4.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();
  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 3.)
  });
  let res = addr
    .send(MidPrice {
      price: 5.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();
  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 4.)
  });
  let res = addr
    .send(MidPrice {
      price: 6.,
      symbol: "".to_owned(),
    })
    .await
    .unwrap();
  assert_matches!(res, MidPriceResponse::MovingAverage(f) => {
    assert_eq!(f, 5.)
  });
}
