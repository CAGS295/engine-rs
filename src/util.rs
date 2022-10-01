use actix::Message;

#[derive(Message)]
#[rtype(result = "f64")]
pub struct Double(pub f64);

#[derive(Message)]
#[rtype(result = "f64")]
pub struct Return(pub f64);

#[derive(Message)]
#[rtype(result = "Option<f64>")]
pub struct SharpeRatio(pub f64);
