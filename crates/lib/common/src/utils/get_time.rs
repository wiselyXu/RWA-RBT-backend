
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{Local, Utc};

pub fn get_current_timestamp_nanos() -> u128 {
    // let start = SystemTime::now();
    // let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went
    // backwards"); since_epoch.as_nanos()

    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

    // let current_timestamp = Utc::now().timestamp();
    // println!("当前时间戳: {}", current_timestamp);
    time
}

// 格式化纳秒时间显示
pub fn format_latency(nanos: u128) -> String {
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{} ns ({:.3} µs)", nanos, nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{} ns ({:.3} ms)", nanos, nanos as f64 / 1_000_000.0)
    } else {
        format!("{} ns ({:.3} s)", nanos, nanos as f64 / 1_000_000_000.0)
    }
}

pub fn format_timestamp_ms(timestamp_ms: i64) -> String {
    let naive = chrono::NaiveDateTime::from_timestamp_millis(timestamp_ms).expect("Invalid timestamp");
    let datetime = chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    let local_time = datetime.with_timezone(&Local);
    local_time.format("%Y-%m-%d %H:%M:%S.%3f %Z").to_string()
}


/// 计算下一个 0ms 或 500ms 的触发时间
fn next_tick_time() -> Instant {
    let now = Utc::now();
    let millis = now.timestamp_subsec_millis();

    // 计算最近的 0ms 或 500ms 触发点
    let wait_millis = if millis % 1000 < 500 {
        500 - (millis % 1000)  // 距离 500ms 的时间
    } else {
        1000 - (millis % 1000) // 距离下个 0ms 的时间
    };

    Instant::now() + Duration::from_millis(wait_millis as u64)
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_get_current_timestamp_nanos() {
        let timestamp = get_current_timestamp_nanos();
        assert!(timestamp > 0);
        assert!(timestamp < u128::MAX);
    }

    #[test]
    fn test_format_latency() {
        assert_eq!(format_latency(500), "500 ns");
        assert_eq!(format_latency(1500), "1500 ns (1.500 µs)");
        assert_eq!(format_latency(1_500_000), "1500000 ns (1.500 ms)");
        assert_eq!(format_latency(1_500_000_000), "1500000000 ns (1.500 s)");
    }

    #[test]
    fn test_hashmap_retain() {
        // 创建一个存储学生分数的 HashMap
        let mut scores: HashMap<String, i32> = HashMap::new();
        scores.insert("Alice".to_string(), 60);
        scores.insert("Bob".to_string(), 85);
        scores.insert("Charlie".to_string(), 45);
        scores.insert("David".to_string(), 90);

        // retain 方法接收一个闭包，该闭包返回 true 表示保留该元素，false 表示移除
        // 参数是 (&K, &mut V)，即 (key的引用, value的可变引用)
        scores.retain(|name, score| {
            // 只保留分数大于等于60的学生
            *score >= 60
        });

        // 验证结果：只剩下及格的学生
        assert_eq!(scores.len(), 3);
        assert!(scores.contains_key("Alice")); // 60分，保留
        assert!(scores.contains_key("Bob")); // 85分，保留
        assert!(!scores.contains_key("Charlie")); // 45分，被移除
        assert!(scores.contains_key("David")); // 90分，保留

        // retain 也可以在保留的同时修改值
        scores.retain(|_name, score| {
            // 所有及格分数加10分奖励
            *score += 10;
            // 返回 true 表示保留所有元素
            true
        });

        // 验证分数都增加了10分
        assert_eq!(scores["Alice"], 70);
        assert_eq!(scores["Bob"], 95);
        assert_eq!(scores["David"], 100);

        // retain 还可以用来根据 key 和 value 的组合条件筛选
        scores.retain(|name, score| {
            // 只保留名字长度大于3且分数小于90的记录
            name.len() > 3 && *score < 90
        });

        // 验证筛选结果
        assert_eq!(scores.len(), 1);
        assert_eq!(scores["Alice"], 70); // 只有 Alice 符合条件
    }
}
