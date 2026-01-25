use std::{collections::BinaryHeap};

use serde_json::{Map, Value};   
pub type JSONObject = serde_json::Map<String, serde_json::Value>;

pub struct MessageBus {
    messages: BinaryHeap<Message>,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus {
            messages: BinaryHeap::new(),
        }
    }

    pub fn schedule_message(
        &mut self,
        sender: &String,
        receiver: MessageReceiver,
        kind: String,
        content: JSONObject,
        receive_at: u64,
    ) {
        let message = Message {
            sender: sender.clone(),
            receiver,
            content,
            kind,
            receive_step: receive_at,
        };

        self.messages.push(message);
    }

    // Retrieve one message scheduled for delivery at the current step
    // Returns None if no messages are deliverable at this step
    pub fn pop_deliverable_message(&mut self, current_time: u64) -> Option<Message> {
        match self.messages.peek() {
            Some(msg) if msg.receive_step <= current_time => Some(self.messages.pop().unwrap()),
            _ => None,
        }
    }

    // Get an iterator over all messages (used for snapshotting)
    pub fn get_messages_iter(&self) -> impl Iterator<Item = &Message> {
        self.messages.iter()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub sender: String,
    pub receiver: MessageReceiver,
    pub content: Map<String, Value>,
    pub kind: String, // Kind of message (e.g., "HealthStatus", "TradeRequest", etc.)
    pub receive_step: u64, // Step at which the message should be received
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MessageReceiver {
    Entity { id: String },               // Entity ID and Component TypeId
    Radius2D { x: f32, y: f32, radius: f32 }, // Broadcast to all components of a given type within radius
}

impl Eq for Message {}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.receive_step == other.receive_step
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap behavior
        other.receive_step.cmp(&self.receive_step)
    }
}

// Commands that entities can issue to the world during their update
#[derive(Debug, Clone)]
pub enum Command {
    RemoveEntity { id: String },
    SendMessage {
        sender: String,
        receiver: MessageReceiver,
        kind: String,
        content: JSONObject,
        delay: u64,
    },
    RecordMetric { name: String, value: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_scheduling_and_delivery() {
        let mut bus = MessageBus::new();
        let sender = "agent".to_string();

        let make_json = |text: &str| {
            let mut obj = serde_json::Map::new();
            obj.insert("text".to_string(), serde_json::Value::String(text.to_string()));
            obj
        };

        bus.schedule_message(
            &sender,
            MessageReceiver::Entity { id: "agent_1".to_string() },
            String::from("Greeting"),
            make_json("Hello"),
            3,
        );
        bus.schedule_message(
            &sender,
            MessageReceiver::Entity { id: "agent_2".to_string() },
            String::from("Greeting"),
            make_json("Hi"),
            3,
        );
        
        bus.schedule_message(
            &sender,
            MessageReceiver::Entity { id: "agent_2".to_string() },
            String::from("Greeting"),
            make_json("Hi, again"),
            2,
        );

        // At step 1, no messages should be deliverable
        assert!(bus.pop_deliverable_message(1).is_none());

        // At step 2, the message scheduled for step 2 should be deliverable
        let msg1 = bus.pop_deliverable_message(2).unwrap();
        assert_eq!(msg1.receive_step, 2);

        // At step 3, the two messages scheduled for step 3 should be deliverable
        let msg2 = bus.pop_deliverable_message(3).unwrap();
        assert_eq!(msg2.receive_step, 3);

        let msg3 = bus.pop_deliverable_message(3).unwrap();
        assert_eq!(msg3.receive_step, 3);

        // No more messages should be deliverable
        assert!(bus.pop_deliverable_message(5).is_none());
    }
}
