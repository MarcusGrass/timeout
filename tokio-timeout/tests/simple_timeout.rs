use std::time::Duration;
use tokio_timeout::timeout;

#[timeout(duration = "1m10s", on_error = "panic")]
pub async fn my_panic_fn() {}

pub enum MyErr {
    Timeout(&'static str),
}

#[timeout(duration = "5ms", on_error = MyErr::Timeout)]
pub async fn my_res_fn() -> Result<String, MyErr> {
    Ok("".to_string())
}

#[timeout(duration = "1ms", on_error = MyErr::Timeout)]
pub async fn my_will_time_out_fn() -> Result<String, MyErr> {
    tokio::time::sleep(core::time::Duration::from_millis(1000)).await;
    Ok("".to_string())
}

const MY_DUR: Duration = Duration::from_millis(1);

#[timeout(duration = crate::MY_DUR, on_error = MyErr::Timeout)]
pub async fn my_will_time_out_const() -> Result<String, MyErr> {
    tokio::time::sleep(core::time::Duration::from_millis(1000)).await;
    Ok("".to_string())
}

#[tokio::test]
async fn smoke_times_out() {
    let err = my_will_time_out_fn().await.err().unwrap();
    assert!(matches!(err, MyErr::Timeout(_)));
}
