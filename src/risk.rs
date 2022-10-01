pub mod sharpe;

use actix::{Actor, Context, Handler, Recipient};

use crate::util::Double;

pub struct Drawdown {
  peak: f64,
  trough: f64,
  max_drawdown: f64,
  subscribers: Vec<Recipient<Double>>,
}

impl Drawdown {
  pub fn new(subscribers: Vec<Recipient<Double>>) -> Self {
    Self {
      peak: std::f64::NEG_INFINITY,
      trough: std::f64::INFINITY,
      max_drawdown: 0.,
      subscribers,
    }
  }
}

impl Actor for Drawdown {
  type Context = Context<Self>;
}

impl Handler<Double> for Drawdown {
  type Result = f64;

  fn handle(&mut self, msg: Double, _ctx: &mut Context<Self>) -> Self::Result {
    if msg.0 > self.peak {
      self.peak = msg.0;
      self.trough = self.peak;
    } else if msg.0 < self.trough {
      self.trough = msg.0;

      let drawdown = self.peak - self.trough;
      self.max_drawdown = self.max_drawdown.max(drawdown);
    }

    for s in &self.subscribers {
      s.do_send(Double(self.max_drawdown));
    }

    self.max_drawdown
  }
}
