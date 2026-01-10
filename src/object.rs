// Trait defining the behavior of smart objects in the simulator
pub trait SmartObjectBehavior {
    fn get_object_type(&self) -> &str;
    fn interact(&self);

    fn list_properties(&self) -> Vec<String>;
    fn list_functions(&self) -> Vec<String>;

    fn call_function(&mut self, function_name: &str, args: Vec<String>) -> Option<String>;
    fn get_property(&self, property_name: &str) -> Option<String>;
    fn set_property(&mut self, property_name: &str, value: String) -> bool;
}