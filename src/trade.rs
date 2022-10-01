use binance::account::*;
use binance::api::*;
use binance::config::Config;
use binance::rest_model::Transaction;
use binance::rest_model::{OrderSide, OrderType, TimeInForce};

async fn buy(
  symbol: &str,
  quantity: f64,
  price: f64,
) -> Result<Transaction, binance::errors::Error> {
  let account: Account = Binance::new_with_env(&Config::testnet());
  let market_buy = OrderRequest {
    symbol: symbol.to_string(),
    quantity: Some(quantity),
    price: Some(price),
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::GTC),
    side: OrderSide::Buy,
    ..OrderRequest::default()
  };

  account.place_order(market_buy).await
}

async fn sell(
  symbol: &str,
  quantity: f64,
  price: f64,
) -> Result<Transaction, binance::errors::Error> {
  let account: Account = Binance::new_with_env(&Config::testnet());
  let market_buy = OrderRequest {
    symbol: symbol.to_string(),
    quantity: Some(quantity),
    price: Some(price),
    order_type: OrderType::Limit,
    time_in_force: Some(TimeInForce::FOK),
    side: OrderSide::Sell,
    ..OrderRequest::default()
  };
  let res = account.place_order(market_buy).await;
  return res;
}

#[cfg(test)]
mod test {
  use super::*;
  use dotenv::dotenv;

  #[actix_rt::test]
  async fn test_buy() {
    dotenv().ok();

    let symbol = "BTCUSDT";
    let quantity = 0.001;
    let price = 19000.;
    let result = buy(symbol, quantity, price).await;
    println!("{:?}", result);
    assert!(result.is_ok());
  }

  #[actix_rt::test]
  async fn test_sell() {
    dotenv().ok();

    let symbol = "BTCUSDT";
    let quantity = 0.001;
    let price = 19000.;
    let result = sell(&symbol, quantity, price).await;
    println!("{:?}", result);
    assert!(result.is_ok());
  }
}
