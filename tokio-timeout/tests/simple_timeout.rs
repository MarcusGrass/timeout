use tokio_timeout::timeout;

#[timeout(duration = "1m10s")]
pub fn my_fn() {}

fn my_dur() {
    
}