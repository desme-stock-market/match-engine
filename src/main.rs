mod engine;
mod kafka;
mod matcher;
mod model;
mod policies;
mod storage;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::engine::MatchEngine;
use crate::kafka::KafkaConsumer;
use crate::model::IncomingOrder;

#[tokio::main]
async fn main() -> Result<()> {
    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let group_id = std::env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "match-engine".to_string());
    let topic = std::env::var("KAFKA_TOPIC").unwrap_or_else(|_| "orders".to_string());

    let (tx, mut rx) = mpsc::channel::<IncomingOrder>(1000);

    let kafka_consumer = KafkaConsumer::new(&brokers, &group_id, &topic)?;

    let consumer_handle = tokio::spawn(async move { kafka_consumer.consume_orders(tx).await });

    let mut match_engine = MatchEngine::new();

    let engine_handle = tokio::spawn(async move {
        while let Some(order) = rx.recv().await {
            let events = match_engine.process(order);

            for event in events {
                println!("Event: {:?}", event);
            }
        }
    });

    tokio::select! {
        _ = consumer_handle => println!("Consumer stopped"),
        _ = engine_handle => println!("Engine stopped"),
    }

    Ok(())
}
