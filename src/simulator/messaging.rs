use std::{any::TypeId, collections::BinaryHeap, time};

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
        receiver: MessageReceiver,
        kind: String,
        content: MessageContent,
        scheduled_at : time::Instant,
        delay: time::Duration,
    ) {
        let message = Message {
            receiver,
            content,
            kind,
            receive_step: scheduled_at + delay,
        };

        self.messages.push(message);
    }

    // Retrieve one message scheduled for delivery at the current step
    // Returns None if no messages are deliverable at this step
    pub fn get_deliverable_message(&mut self, current_time: time::Instant) -> Option<Message> {
        match self.messages.peek() {
            Some(msg) if msg.receive_step <= current_time => Some(self.messages.pop().unwrap()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub receiver: MessageReceiver,
    pub content: MessageContent,
    pub kind: String, // Kind of message (e.g., "HealthStatus", "TradeRequest", etc.)
    pub receive_step: time::Instant, // Step at which the message should be received
}

#[derive(Debug, Clone)]
pub enum MessageReceiver {
    None,
    Entity { id: u32 },                       // Entity ID and Component TypeId
    Radius2D { x: f32, y: f32, radius: f32 }, // Broadcast to all components of a given type within radius
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageContent {
    Text(String),
    Data(Vec<u8>),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_scheduling_and_delivery() {
        let mut bus = MessageBus::new();
        let mut current_time = time::Instant::now();

        bus.schedule_message(
            MessageReceiver::Entity { id: 1 },
            String::from("Greeting"),
            MessageContent::Text("Hello".to_string()),
            current_time,
            time::Duration::from_secs(3),
        );
        bus.schedule_message(
            MessageReceiver::Entity { id: 2 },
            String::from("Greeting"),
            MessageContent::Text("Hi".to_string()),
            current_time + time::Duration::from_secs(1),
            time::Duration::from_secs(3),
        );
        bus.schedule_message(
            MessageReceiver::Entity { id: 2 },
            String::from("Greeting"),
            MessageContent::Text("Hi, again".to_string()),
            current_time + time::Duration::from_secs(2),
            time::Duration::from_secs(2),
        );

        current_time += time::Duration::from_secs(2);
        // At step 2, no messages should be deliverable
        assert!(bus.get_deliverable_message(current_time).is_none());

        current_time += time::Duration::from_secs(1);
        // At step 3, the first message should be deliverable
        let msg1 = bus.get_deliverable_message(current_time).unwrap();
        assert_eq!(msg1.content, MessageContent::Text("Hello".to_string()));

        current_time += time::Duration::from_secs(1);
        // At step 4, the second message should be deliverable
        let msg2 = bus.get_deliverable_message(current_time).unwrap();
        assert_eq!(msg2.content, MessageContent::Text("Hi".to_string()));

        current_time += time::Duration::from_secs(1);
        // At step 4, the third message should also be deliverable
        let msg3 = bus.get_deliverable_message(current_time).unwrap();
        assert_eq!(msg3.content, MessageContent::Text("Hi, again".to_string()));

        current_time += time::Duration::from_secs(1);
        // No more messages should be deliverable
        assert!(bus.get_deliverable_message(current_time).is_none());
    }
}
