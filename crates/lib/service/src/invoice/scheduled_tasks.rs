use chrono::{Utc, Duration, Local, NaiveTime};
use tokio::time::{self, sleep};
use crate::invoice::InvoiceService;
use log::{info, error};
use std::sync::Arc;

// 设置定时任务
pub fn setup_scheduled_tasks(invoice_service: Arc<InvoiceService>) {
    // 启动每日计息任务
    tokio::spawn(daily_interest_calculation_task(invoice_service.clone()));
    
    // 启动到期兑付任务
    tokio::spawn(maturity_payment_task(invoice_service));
}

// 每日计息任务
async fn daily_interest_calculation_task(invoice_service: Arc<InvoiceService>) {
    // 设置每日运行时间（例如UTC 0:10，给前一天的交易留出时间完成）
    let target_time = NaiveTime::from_hms_opt(0, 10, 0).unwrap();
    
    loop {
        // 等待到指定时间
        let now = Local::now();
        let next_run = if now.time() >= target_time {
            // 如果当前时间已过今天的计划时间，则安排在明天的同一时间
            now.date_naive().succ_opt().unwrap().and_time(target_time)
        } else {
            // 否则安排在今天的计划时间
            now.date_naive().and_time(target_time)
        };
        
        // 计算需要等待的时间
        let wait_duration = next_run.and_local_timezone(Local).unwrap() - now;
        info!("下一次计息任务将在 {} 运行", next_run);
        sleep(wait_duration.to_std().unwrap()).await;
        
        // 获取昨天的日期（计算的是前一天的利息）
        let yesterday = Utc::now().date_naive() - Duration::days(1);
        
        info!("开始计算 {} 的每日利息", yesterday);
        
        // 执行计息任务
        match invoice_service.calculate_daily_interest_for_date(yesterday).await {
            Ok(success_count) => {
                info!(
                    "计息任务完成: 成功处理 {} 条持仓记录", 
                    success_count
                );
            },
            Err(e) => {
                error!("执行计息任务失败: {:?}", e);
            }
        }
        
        // 避免在同一秒内多次执行
        sleep(time::Duration::from_secs(1)).await;
    }
}

// 到期兑付任务
async fn maturity_payment_task(invoice_service: Arc<InvoiceService>) {
    // 设置每日运行时间（例如UTC 1:00，在计息任务之后）
    let target_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();
    
    loop {
        // 等待到指定时间
        let now = Local::now();
        let next_run = if now.time() >= target_time {
            // 如果当前时间已过今天的计划时间，则安排在明天的同一时间
            now.date_naive().succ_opt().unwrap().and_time(target_time)
        } else {
            // 否则安排在今天的计划时间
            now.date_naive().and_time(target_time)
        };
        
        // 计算需要等待的时间
        let wait_duration = next_run.and_local_timezone(Local).unwrap() - now;
        info!("下一次到期兑付任务将在 {} 运行", next_run);
        sleep(wait_duration.to_std().unwrap()).await;
        
        // 获取当天日期（检查今天到期的票据）
        let today = Utc::now().date_naive();
        
        info!("开始处理 {} 到期的票据", today);
        
        // 执行到期兑付任务
        match invoice_service.process_maturity_payments_for_date(today).await {
            Ok(success_count) => {
                info!("到期兑付任务完成: 成功处理 {} 条持仓记录", success_count);
            },
            Err(e) => {
                error!("执行到期兑付任务失败: {:?}", e);
            }
        }
        
        // 避免在同一秒内多次执行
        sleep(time::Duration::from_secs(1)).await;
    }
}
