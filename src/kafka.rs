use anyhow::Result;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    message::Message,
    ClientConfig,
};
use tokio::sync::mpsc;

use crate::model::IncomingOrder;

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(brokers: &str, group_id: &str, topic: &str) -> Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(&[topic])?;

        Ok(KafkaConsumer { consumer })
    }

    pub async fn consume_orders(&self, tx: mpsc::Sender<IncomingOrder>) -> Result<()> {
        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<IncomingOrder>(payload) {
                            Ok(order) => {
                                if tx.send(order).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize order: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Kafka error: {}", e);
                }
            }
        }
        Ok(())
    }
}
