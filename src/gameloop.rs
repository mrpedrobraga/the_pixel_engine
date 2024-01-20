use std::time::{Duration, Instant};

pub async fn game_loop<F>(fps: u64, update_fn: F) -> ()
where
    F: Fn(Duration) -> (),
{
    let target_dt_ns = 1_000_000_000 / fps;
    let target_dt_ns_d = Duration::from_nanos(target_dt_ns);
    let mut dt = target_dt_ns_d;

    loop {
        let t0 = Instant::now();

        async {
            update_fn(dt);
        }
        .await;

        let t1 = t0.elapsed();

        if t1 < target_dt_ns_d {
            let resting_time = target_dt_ns_d - t1;
            std::thread::sleep(resting_time);
        }

        dt = t0.elapsed();
    }
}
