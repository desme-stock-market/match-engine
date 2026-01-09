use crate::model::{EngineEvent, IncomingOrder};

pub struct Log;

impl Log {
    pub fn order(count: u64, order: &IncomingOrder) {
        println!("[ORDER #{}] {:?} {} @ {:?}", 
            count, order.side, order.amount as f64 / 1_000_000.0, order.order_type);
    }

    pub fn events(events: &[EngineEvent]) {
        for event in events {
            match event {
                EngineEvent::TradeExecuted(trade) => {
                    println!("  → TRADE: {} tokens @ ${:.4} (trade_id: {})", 
                        trade.amount as f64 / 1_000_000.0,
                        trade.price.mantissa() as f64 / 10_f64.powi(trade.price.scale() as i32),
                        trade.trade_id);
                }
                EngineEvent::OrderPlaced { .. } => println!("  → ORDER_PLACED"),
                EngineEvent::OrderCancelled { order_id, reason, .. } => {
                    println!("  → ORDER_CANCELLED: {} ({})", order_id, reason);
                }
                EngineEvent::OrderRejected { order_id, reason } => {
                    println!("  → ORDER_REJECTED: {} ({})", order_id, reason);
                }
            }
        }
    }
}
