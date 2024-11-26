pub mod cot;
pub mod stanag_4586;

/// Trait implemented by protocols for generating telemetry messages.
pub trait TelemMsg {
    fn from_coords(lat: f64, lon: f64, alt_hae: f32) -> Self;
    fn with_agent_id(self, agent_id: &str) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}
