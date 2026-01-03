mod engine;
mod kafka;
mod matcher;
mod model;
mod policies;
mod storage;

use anyhow::Result;
use smallvec::SmallVec;
use tokio::sync::mpsc;

use crate::engine::MatchEngine;
use crate::kafka::{KafkaConsumer, KafkaProducer};
use crate::model::{EngineEvent, IncomingOrder};

#[tokio::main]
async fn main() -> Result<()> {
    let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let group_id = std::env::var("KAFKA_GROUP_ID").unwrap_or_else(|_| "match-engine".to_string());
    let consumer_topic =
        std::env::var("KAFKA_CONSUMER_TOPIC").unwrap_or_else(|_| "orders".to_string());
    let producer_topic =
        std::env::var("KAFKA_PRODUCER_TOPIC").unwrap_or_else(|_| "trades".to_string());

    let (tx, mut rx) = mpsc::channel::<IncomingOrder>(1000);

    let kafka_consumer = KafkaConsumer::new(&brokers, &group_id, &consumer_topic)?;
    let kafka_producer = KafkaProducer::new(&brokers)?;

    let consumer_handle = tokio::spawn(async move { kafka_consumer.consume_orders(tx).await });

    let mut match_engine = MatchEngine::new();

    let engine_handle = tokio::spawn(async move {
        let mut events_batch: SmallVec<[EngineEvent; 16]> = SmallVec::new();
        let batch_size = 16;
        let flush_interval = tokio::time::Duration::from_millis(100);
        let mut flush_timer = tokio::time::interval(flush_interval);

        loop {
            tokio::select! {
                Some(order) = rx.recv() => {
                    let events = match_engine.process(order);
                    events_batch.extend(events);

                    if events_batch.len() >= batch_size {
                        if let Err(e) = kafka_producer.send_events(&producer_topic, events_batch.drain(..).collect()).await {
                            eprintln!("Failed to send events batch: {}", e);
                        }
                    }
                }
                _ = flush_timer.tick() => {
                    if !events_batch.is_empty() {
                        if let Err(e) = kafka_producer.send_events(&producer_topic, events_batch.drain(..).collect()).await {
                            eprintln!("Failed to send events batch: {}", e);
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = consumer_handle => println!("Consumer stopped"),
        _ = engine_handle => println!("Engine stopped"),
    }

    Ok(())
}
