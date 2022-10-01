use engine_rs::risk::Drawdown;
use engine_rs::util::Double;

use actix::Actor;

#[actix_rt::test]
async fn test() {
  let addr1 = Drawdown::new(vec![]).start();

  let res = addr1.send(Double(0.)).await.unwrap();

  assert_eq!(res, 0.);

  let res = addr1.send(Double(1.)).await.unwrap();

  assert_eq!(res, 0.);

  let res = addr1.send(Double(0.5)).await.unwrap();

  assert_eq!(res, 0.5);

  let res = addr1.send(Double(0.)).await.unwrap();

  assert_eq!(res, 1.);
}
