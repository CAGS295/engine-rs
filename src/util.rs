use actix::Message;

#[derive(Message)]
#[rtype(result = "f64")]
pub struct Double(pub f64);
