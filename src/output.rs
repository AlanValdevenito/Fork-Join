use std::collections::HashMap;

use crate::Player;
use crate::Weapon;

/// Structure used to store processed information
///
#[derive(serde::Serialize, Debug)]
pub struct Output {
    pub padron: u32,
    /// Top 10 players who produced the most kills
    pub top_killers: HashMap<String, Player>,
    /// Top 10 weapons that produced the most kills
    pub top_weapons: HashMap<String, Weapon>,
}

impl Output {
    /// Writes the processed information that the structure has to a file
    ///
    /// # Arguments
    ///
    /// * 'output_file_name': File name
    ///
    pub fn write_to_file(&self, output_file_name: &str) -> std::io::Result<()> {
        let output_json = match serde_json::to_string_pretty(&self) {
            Ok(output_json) => output_json,
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to parse JSON",
                ));
            }
        };

        if let Err(e) = std::fs::write(output_file_name, &output_json) {
            eprintln!("Error: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write file",
            ));
        };

        Ok(())
    }
}
