use anyhow::Result;
use rdkafka::{
    ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use smallvec::SmallVec;
use tokio::sync::mpsc;

use crate::model::{EngineEvent, IncomingOrder};

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

pub struct KafkaProducer {
    producer: FutureProducer,
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

impl KafkaProducer {
    pub fn new(brokers: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("acks", "all")
            .create()?;

        Ok(KafkaProducer { producer })
    }

    pub async fn send_events(
        &self,
        topic: &str,
        events: SmallVec<[EngineEvent; 16]>,
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        for event in events {
            let payload = serde_json::to_vec(&event)?;
            let record = FutureRecord::to(topic).key("").payload(&payload);

            self.producer
                .send(record, None)
                .await
                .map_err(|(e, _)| anyhow::anyhow!("Failed to send event to Kafka: {}", e))?;
        }

        Ok(())
    }
}
