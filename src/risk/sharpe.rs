use crate::algos::single_pass::{mean, mean_centered_sum_squared, sum};
use crate::util::{Return, SharpeRatio};
use actix::{Actor, Context, Handler, Recipient};
use core::marker::PhantomData;

///Computes the mean average
///Computes the standard deviation
///Calculates the sharpe ratio
pub struct Sharpe {
  mean: f64,
  s_squared: f64,
  cold_count: u32,
  _p: PhantomData<f64>,
  init_buffer: Option<Vec<f64>>,

  subscribers: Vec<Recipient<SharpeRatio>>,
  window_size: u32,
}

impl Sharpe {
  pub fn new(
    subscribers: Vec<Recipient<SharpeRatio>>,
    window_size: u32,
  ) -> Self {
    Sharpe {
      mean: 0.,
      s_squared: 0.,
      cold_count: 0,
      _p: PhantomData,
      init_buffer: None,
      subscribers,
      window_size,
    }
  }

  pub fn update(&mut self, new: f64) -> Option<f64> {
    match self.cold_count {
      count if count > self.window_size => {
        let mean_1 = self.mean;
        self.mean = mean(self.mean, new, self.window_size);
        self.s_squared += (new - mean_1) * (new - self.mean);
        Some(
          self.mean
            / (f64::sqrt(self.s_squared) / ((self.window_size - 1) as f64)),
        )
      }
      count if count == self.window_size => {
        let x = self.init_buffer.take().expect("returns");
        self.mean = sum(x.iter().copied()) / count as f64;
        self.s_squared = mean_centered_sum_squared(x.into_iter(), self.mean);
        self.cold_count = count + 1;
        Some(
          self.mean
            / (f64::sqrt(self.s_squared) / ((self.window_size - 1) as f64)),
        )
      }
      //stdev
      count => {
        let mut buffer = self
          .init_buffer
          .take()
          .unwrap_or_else(|| Vec::with_capacity(self.window_size as usize));
        buffer.push(new);
        self.init_buffer.replace(buffer);
        self.cold_count = count + 1;
        None
      }
    }
  }
}

impl Actor for Sharpe {
  type Context = Context<Self>;
}

impl Handler<Return> for Sharpe {
  //a sharpe ratio
  type Result = f64;

  fn handle(&mut self, msg: Return, _ctx: &mut Self::Context) -> Self::Result {
    if let Some(ratio) = self.update(msg.0) {
      for s in &self.subscribers {
        s.do_send(SharpeRatio(ratio));
      }
    }
    msg.0
  }
}

//test mean of 0;
//test window_size upper bound

#[cfg(test)]
pub mod tests {
  use super::*;

  #[actix_rt::test]
  async fn dividend_is_zero() {
    let sharpe_actor = Sharpe::new(vec![], 1);
    let addr1 = sharpe_actor.start();
    addr1.send(Return(1.)).await.unwrap();
  }
}
