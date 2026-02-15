
pub trait Scripting {
    // Update the script state and return any commands to be executed
    fn update(&mut self, simulation_time: u64) -> Result<Vec<Command>, CoreError>;
    // Set the internal state of the script from a serialized JSON object
    fn set_state(&mut self, state: messaging::JSONObject) -> Result<(), CoreError>;
    // Get the internal state of the script as a serialized JSON object
    fn get_state(&self) -> Result<JSONObject, CoreError>;
    // Push a message to the script's incoming message queue
    pub fn push_message(&mut self, msg: Message);
}