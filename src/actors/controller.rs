use actix::{Actor, Addr, Context, Handler, Recipient};
use std::collections::HashMap;
use trade::Buy;

pub struct Controller {
  scaling_coefficient: f64,
  subscribers: Vec<Recipient<ControllerCommand>>,
}

impl Controller {
  pub fn new(subscribers: Vec<Recipient<ControllerCommand>>) -> Self {
    Self {
      scaling_coefficient: 1.,
      subscribers,
    }
  }
}

impl Actor for Controller {
  type Context = Context<Self>;
}

impl Handler<PolicyDecision> for Controller {
  type Result = ControllerCommand;

  fn handle(
    &mut self,
    msg: PolicyDecision,
    ctx: &mut Context<Self>,
  ) -> Self::Result {
    let artificial_spread_coefficient;
    match msg {
      PolicyDecision::BuyAction => {
        artificial_spread_coefficient = 1 / scaling_coefficient
      }
      PolicyDecision::SellAction => {
        artificial_spread_coefficient = scaling_coefficient
      }
      PolicyDecision::HoldAction => artificial_spread_coefficient = 1,
    }
    for s in &self.subscribers {
      s.do_send(ControllerCommand(artificial_spread_coefficient));
    }

    artificial_spread_coefficient
  }
}
