/// Structure used to store processed weapon information
///
#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct Weapon {
    /// Percentage of total deaths caused by the weapon
    pub deaths_percentage: f64,
    /// Average distance between murdered and victim for weapon
    pub average_distance: f64,
}
