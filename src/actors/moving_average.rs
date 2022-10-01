use actix::{Actor, Context, Handler, MessageResult, Recipient};

use crate::util::MovingAverageMessage;

// Moving average is 0 if number of received messages
// is not a multiple of the interval_length
pub struct MovingAverage {
  moving_average: f64,
  ring_buffer: Vec<f64>,
  interval_length: usize,
  subscribers: Vec<Recipient<MovingAverageMessage>>,
}

impl MovingAverage {
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

impl Actor for MovingAverage {
  type Context = Context<Self>;
}

impl Handler<MovingAverageMessage> for MovingAverage {
  type Result = MessageResult<MovingAverageMessage>;

  fn handle(
    &mut self,
    msg: MovingAverageMessage,
    _ctx: &mut Context<Self>,
  ) -> Self::Result {
    let empty_buffer_length =
      self.ring_buffer.iter().filter(|&n| *n == 0.).count();

    if empty_buffer_length > 1 {
      self.ring_buffer[self.interval_length - empty_buffer_length] = msg.0;

      MessageResult(0f64)
    } else if empty_buffer_length == 1 {
      self.ring_buffer[self.interval_length - empty_buffer_length] = msg.0;
      self.moving_average =
        self.ring_buffer.iter().sum::<f64>() / self.interval_length as f64;

      for s in &self.subscribers {
        s.do_send(MovingAverageMessage(self.moving_average));
      }

      MessageResult(self.moving_average)
    } else {
      self.ring_buffer.remove(0);
      self.ring_buffer.push(msg.0);
      self.moving_average =
        self.ring_buffer.iter().sum::<f64>() / self.interval_length as f64;

      for s in &self.subscribers {
        s.do_send(MovingAverageMessage(self.moving_average));
      }
      MessageResult(self.moving_average)
    }
  }
}

#[actix_rt::test]
async fn positive() {
  let addr = MovingAverage::new(3, vec![]).start();

  let res = addr.send(MovingAverageMessage(1.)).await.unwrap();
  assert_eq!(res, 0.);
  let res = addr.send(MovingAverageMessage(2.)).await.unwrap();
  assert_eq!(res, 0.);
  let res = addr.send(MovingAverageMessage(3.)).await.unwrap();
  assert_eq!(res, 2.);
  let res = addr.send(MovingAverageMessage(4.)).await.unwrap();
  assert_eq!(res, 3.);
  let res = addr.send(MovingAverageMessage(5.)).await.unwrap();
  assert_eq!(res, 4.);
  let res = addr.send(MovingAverageMessage(6.)).await.unwrap();
  assert_eq!(res, 5.);
}
