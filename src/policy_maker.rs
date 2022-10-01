use std::os::unix::raw::time_t;
use crate::policy_maker::PolicyDecision::Buy;
use crate::trade;
use crate::trade::{Buy, Hold, Sell};

pub struct PolicyMaker {
    moving_average_stream: u64,
    true_price_stream: u64,
}

// PolicyFrame is a snapshot in time, containing all parameters
// necessary to make a policy decision
pub struct PolicyFrame {
    symbol: String,
    moving_average_gradient: f32,
    true_price_gradient: f32,
    moving_average_value: f64,
    true_price_value: f64,
    prev_decision: PolicyDecision,
}

enum PolicyDecision {
    BuyAction(Buy),
    SellAction(Sell),
    HoldAction(Hold),
}

// pub struct BuyAction {
//     symbol: &'static str,
//     quantity: f64,
//     timestamp: time_t,
// }
//
// pub struct SellAction {
//     symbol: &'static str,
//     quantity: f64,
//     timestamp: time_t,
// }
//
// pub struct HoldAction {
//     symbol: &'static str,
//     timestamp: time_t,
// }

impl PolicyMaker {
    async fn run(self) {
        let frame = PolicyFrame{
            moving_average_gradient: 0.0,
            true_price_gradient: 0.0,
            moving_average_value: 0.0,
            true_price_value: 0.0
        };
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
    fn make_policy_decision(self, frame: PolicyFrame) -> PolicyDecision {
        return if frame.moving_average_gradient > 0.0 && frame.true_price_gradient > 0.0 &&
            frame.moving_average_value < frame.true_price_value &&
            !(matches!(frame.prev_decision, PolicyAction::Buy)) {
            PolicyDecision::BuyAction(Buy {
                symbol: frame.symbol,
                quantity: 0.1,
                price: frame.true_price_value,
                timestamp: Utc.timestamp(),
            })
        } else if frame.moving_average_gradient < 0.0 && frame.true_price_gradient < 0.0 &&
            frame.moving_average_value > frame.true_price_value &&
            !(matches!(frame.prev_decision, PolicyAction::Sell)) {
            PolicyDecision::SellAction(Sell {
                symbol: frame.symbol,
                quantity: 0.1,
                price: frame.true_price_value,
                timestamp: Utc.timestamp(),
            })
        } else {
            PolicyDecision::HoldAction(Hold {
                symbol: frame.symbol,
                timestamp: Utc.timestamp(),
            })
        }
    }
}

