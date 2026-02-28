use std::collections::HashMap;

/// Structure used to store processed player information
///
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct Player {
    /// Total number of kills executed by the player
    pub deaths: u32,
    /// Percentage of use with respect to the total deaths caused by that player
    pub weapons_percentage: HashMap<String, f64>,
}
