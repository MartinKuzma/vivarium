use std::collections::BinaryHeap;

pub struct MessageBus {
    messages: BinaryHeap<Message>,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus {
            messages: BinaryHeap::new(),
        }
    }

    pub fn schedule_message(&mut self, sender_id: u32, recipient_id: u32, content: String, current_step: u32, delay: u32) {
        let message = Message {
            sender_id,
            recipient_id,
            content,
            sent_step: current_step,
            receive_step: current_step + delay,
        };
        
        self.messages.push(message);
    }

    // Retrieve one message scheduled for delivery at the current step
    // Returns None if no messages are deliverable at this step
    pub fn get_deliverable_message(&mut self, current_step: u32) -> Option<Message> {
        match self.messages.peek() {
            Some(msg) if msg.receive_step <= current_step => Some(self.messages.pop().unwrap()),
            _ => None,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Message {
    pub sender_id: u32, // ID of the sending agent
    pub recipient_id: u32,
    pub content: String, 
    pub sent_step: u32,    // Step at which the message was sent, for tracking
    pub receive_step: u32, // Step at which the message should be received
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
        bus.schedule_message(1, 2, "Hello".to_string(), 0, 3);
        bus.schedule_message(2, 1, "Hi".to_string(), 1, 2);
        bus.schedule_message(2, 1, "Hi, again".to_string(), 1, 2);

        // At step 2, no messages should be deliverable
        assert!(bus.get_deliverable_message(2).is_none());

        // At step 3, the first message should be deliverable
        let msg1 = bus.get_deliverable_message(3).unwrap();
        assert_eq!(msg1.content, "Hello");

        // At step 4, the second message should be deliverable
        let msg2 = bus.get_deliverable_message(4).unwrap();
        assert_eq!(msg2.content, "Hi");

        // At step 4, the third message should also be deliverable
        let msg3 = bus.get_deliverable_message(4).unwrap();
        assert_eq!(msg3.content, "Hi, again");

        // No more messages should be deliverable
        assert!(bus.get_deliverable_message(5).is_none());
    }
}
