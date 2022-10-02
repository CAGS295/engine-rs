//use crate::*;

use dotenv::dotenv;
use engine_rs::{
  actors::{mid_price::MidPriceActor, moving_average::MovingAverageActor},
  binance_websocket::BinanceIngestor,
  policy_maker::PolicyMakerActor,
  trade::TradeActor,
  Actor,
};

#[actix::main]
async fn main() {
  dotenv().ok();
  let trade_actor = TradeActor::new().start();

  let policy_maker_actor =
    PolicyMakerActor::new(vec![trade_actor.recipient()]).start();

  let moving_avg_actor =
    MovingAverageActor::new(3, vec![policy_maker_actor.clone().recipient()])
      .start();

  let midprice_rcp = MidPriceActor::new(vec![
    moving_avg_actor.clone().recipient(),
    policy_maker_actor.clone().recipient(),
  ])
  .start()
  .recipient();

  let st = BinanceIngestor::new(vec![midprice_rcp], vec![]);
  actix::spawn(st.run()).await.unwrap();
}
