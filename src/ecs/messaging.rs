use std::{any::TypeId, collections::BinaryHeap};

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
        content: MessageContent,
        current_step: u32,
        delay: u32,
    ) {
        let message = Message {
            receiver,
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
    pub receiver : MessageReceiver,
    pub content: MessageContent,
    pub sent_step: u32,    // Step at which the message was sent, for tracking
    pub receive_step: u32, // Step at which the message should be received
}

#[derive(Debug, Clone)]
pub enum MessageReceiver {
    None,
    Entity{id : u32, component_type: TypeId}, // Entity ID and Component TypeId
    Component(TypeId),  // Broadcast to all components of a given type
    Radius2D { x: f32, y: f32, radius: f32, component_type: TypeId }, // Broadcast to all components of a given type within radius
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

// pub struct MessageBuilder {
//     sender_id: u32,
//     content: MessageContent,
//     delay: u32,
//     receiver: MessageReceiver,
// }

// impl MessageBuilder {
//     pub fn new(sender_id: u32, content: MessageContent) -> Self {
//         MessageBuilder {
//             sender_id,
//             content,
//             delay: 0,
//             receiver: MessageReceiver::None,
//         }
//     }

//     pub fn delay(mut self, delay: u32) -> Self {
//         self.delay = delay;
//         self
//     }

//     pub fn to_entity(mut self, entity_id: u32, component_type: TypeId) -> Self {
//         self.receiver = MessageReceiver::Entity { id: entity_id, component_type };
//         self
//     }

//     pub fn to_component_type(mut self, component_type: TypeId) -> Self {
//         self.receiver = MessageReceiver::Component(component_type);
//         self
//     }

//     pub fn to_radius_2d(mut self, x: f32, y: f32, radius: f32, component_type: TypeId) -> Self {
//         self.receiver = MessageReceiver::Radius2D { x, y, radius, component_type };
//         self
//     }

//     pub fn build(self, current_step: u32) -> Message {
//         Message {
//             sender_id: self.sender_id,
//             receiver: self.receiver,
//             content: self.content,
//             sent_step: current_step,
//             receive_step: current_step + self.delay,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_scheduling_and_delivery() {
        let mut bus = MessageBus::new();
        bus.schedule_message(
            MessageReceiver::Entity { id: 1, component_type: TypeId::of::<()>() },
            MessageContent::Text("Hello".to_string()),
            0,
            3,
        );
        bus.schedule_message(
            MessageReceiver::Entity { id: 2, component_type: TypeId::of::<()>() },
            MessageContent::Text("Hi".to_string()),
            1, 3
        );
        bus.schedule_message(
            MessageReceiver::Entity { id: 2, component_type: TypeId::of::<()>() },
            MessageContent::Text("Hi, again".to_string()),
            2,
            2,
        );

        // At step 2, no messages should be deliverable
        assert!(bus.get_deliverable_message(2).is_none());

        // At step 3, the first message should be deliverable
        let msg1 = bus.get_deliverable_message(3).unwrap();
        assert_eq!(msg1.content, MessageContent::Text("Hello".to_string()));

        // At step 4, the second message should be deliverable
        let msg2 = bus.get_deliverable_message(4).unwrap();
        assert_eq!(msg2.content, MessageContent::Text("Hi".to_string()));

        // At step 4, the third message should also be deliverable
        let msg3 = bus.get_deliverable_message(4).unwrap();
        assert_eq!(msg3.content, MessageContent::Text("Hi, again".to_string()));

        // No more messages should be deliverable
        assert!(bus.get_deliverable_message(5).is_none());
    }
}
