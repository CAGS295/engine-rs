use actix::{Actor, Context, Handler, Recipient};

use crate::util::Double;

// Moving average is 0 if number of received messages
// is not a multiple of the interval_length
pub struct MovingAverage {
  moving_average: f64,
  partial_sum: f64,
  current_interval: usize,
  interval_length: usize,
  subscribers: Vec<Recipient<Double>>,
}

impl MovingAverage {
  pub fn new(
    interval_length: usize,
    subscribers: Vec<Recipient<Double>>,
  ) -> Self {
    Self {
      moving_average: 0.,
      partial_sum: 0.,
      current_interval: 0,
      interval_length,
      subscribers,
    }
  }
}

impl Actor for MovingAverage {
  type Context = Context<Self>;
}

impl Handler<Double> for MovingAverage {
  type Result = f64;

  fn handle(&mut self, msg: Double, _ctx: &mut Context<Self>) -> Self::Result {
    self.current_interval += 1;
    self.current_interval %= self.interval_length;
    self.partial_sum += msg.0;

    if self.current_interval == 1 {
      self.partial_sum = msg.0;
    }

    if self.current_interval == 0 {
      self.partial_sum /= self.interval_length as f64;

      for s in &self.subscribers {
        s.do_send(Double(self.partial_sum));
      }

      self.moving_average = self.partial_sum;
    }

    self.moving_average
  }
}

#[actix_rt::test]
async fn positive() {
  let addr = MovingAverage::new(3, vec![]).start();

  let res = addr.send(Double(1.)).await.unwrap();
  assert_eq!(res, 0.);
  let res = addr.send(Double(2.)).await.unwrap();
  assert_eq!(res, 0.);
  let res = addr.send(Double(3.)).await.unwrap();
  assert_eq!(res, 2.);
  let res = addr.send(Double(4.)).await.unwrap();
  assert_eq!(res, 2.);
  let res = addr.send(Double(5.)).await.unwrap();
  assert_eq!(res, 2.);
  let res = addr.send(Double(6.)).await.unwrap();
  assert_eq!(res, 5.);
}