use engine_rs::moving_average::MovingAverage;
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

  let addr2 = MovingAverage::new(5, vec![]).start();

  let res = addr2.send(Double(1.)).await.unwrap();

  assert_eq!(res, 0.);

  addr2.send(Double(2.)).await.unwrap();
  addr2.send(Double(3.)).await.unwrap();
  addr2.send(Double(4.)).await.unwrap();
  let res = addr2.send(Double(5.)).await.unwrap();

  assert_eq!(res, 3.);

  let res = addr2.send(Double(2.)).await.unwrap();

  assert_eq!(res, 0.);
}
