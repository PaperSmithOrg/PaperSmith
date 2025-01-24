use chrono::prelude::*;

pub fn calculate(word_count: usize, start_time: Option<DateTime<Local>>) -> f64 {
    if let Some(start) = start_time {
        let elapsed = Local::now() - start;
        if elapsed.num_seconds() > 0 {
            #[allow(clippy::cast_precision_loss)]
            return (word_count as f64 / elapsed.num_seconds() as f64) * 60.0;
        }
    }
    0.0
}
