use std::collections::HashMap;

use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};

use std::fs::{read_dir, File};

use std::path::PathBuf;

use std::io::{BufRead, BufReader};

use crate::Output;
use crate::Player;
use crate::Weapon;

/// Sort the elements based on the number of deaths
/// In case of a tie, sort alphabetically
///
/// # Arguments
///
/// * 'weapons': Processed weapons information
///
/// # Returns
///
/// HashMap that store top 10 weapons with the highest number of deaths
pub fn get_top_weapons(weapons: HashMap<String, Weapon>, n: usize) -> HashMap<String, Weapon> {
    let mut weapons: Vec<(String, Weapon)> = weapons.into_iter().collect();

    weapons.sort_by(|a, b| {
        match b.1.deaths_percentage.partial_cmp(&a.1.deaths_percentage) {
            Some(order) => order,
            None => a.0.cmp(&b.0),
        }
        .then(a.0.cmp(&b.0))
    });

    weapons.into_iter().take(n).collect()
}

/// Sort the elements based on the number of deaths
/// In case of a tie, sort alphabetically
///
/// # Arguments
///
/// * 'players': Processed players information
///
/// # Returns
///
/// HashMap that store top 10 players with the highest number of deaths
pub fn get_top_killers(players: HashMap<String, Player>, n: usize) -> HashMap<String, Player> {
    let mut players: Vec<(String, Player)> = players.into_iter().collect();

    players.sort_by(|a, b| b.1.deaths.cmp(&a.1.deaths).then(a.0.cmp(&b.0)));

    let mut players: HashMap<String, Player> = players.into_iter().take(n).collect();

    for player in players.values_mut() {
        let deaths = player.deaths as f64;

        for percentage in player.weapons_percentage.values_mut() {
            *percentage = ((*percentage / deaths * 100.0) * 100.0).round() / 100.0;
        }

        let mut top_weapons: Vec<(String, f64)> = player.weapons_percentage.drain().collect();

        top_weapons.sort_by(|a, b| {
            if let Some(order) = b.1.partial_cmp(&a.1) {
                order.then(a.0.cmp(&b.0))
            } else {
                std::cmp::Ordering::Equal
            }
        });

        player.weapons_percentage = top_weapons.into_iter().take(3).collect::<HashMap<_, _>>();
    }

    players
}

/// Process weapon statistics by calculating death percentage and average distance
///
/// # Arguments
///
/// * 'weapons_stats': Weapon statistics
///
/// # Returns
///
/// HashMap with processed weapon statistics
pub fn process_weapon_stats(
    weapons_stats: HashMap<String, (u32, f64, f64)>,
) -> HashMap<String, Weapon> {
    let total_deaths: u32 = weapons_stats.values().map(|(deaths, _, _)| deaths).sum();

    let mut weapons: HashMap<String, Weapon> = HashMap::new();

    for (weapon_name, (deaths, distance, count)) in weapons_stats {
        let deaths_percentage =
            ((deaths as f64 / total_deaths as f64 * 100.0) * 100.0).round() / 100.0;
        let average_distance = ((distance / count) * 100.0).round() / 100.0;

        weapons.insert(
            weapon_name,
            Weapon {
                deaths_percentage,
                average_distance,
            },
        );
    }

    weapons
}

/// Merge two structures that contain processed information
///
/// # Arguments
///
/// * 'weapons1': First structure that store information about weapons
/// * 'weapons2': Second structure that store information about weapons
///
pub fn merge_killers(killers1: &mut HashMap<String, Player>, killers2: HashMap<String, Player>) {
    killers2.iter().for_each(|(killer_str_2, killer_2)| {
        killers1
            .entry(killer_str_2.to_string())
            .and_modify(|killer_1| {
                killer_1.deaths += killer_2.deaths;

                killer_2
                    .weapons_percentage
                    .iter()
                    .for_each(|(weapon, percentage_2)| {
                        killer_1
                            .weapons_percentage
                            .entry(weapon.to_string())
                            .and_modify(|percentage_1| {
                                *percentage_1 += percentage_2;
                            })
                            .or_insert(*percentage_2);
                    })
            })
            .or_insert(killer_2.clone());
    })
}

/// Merge two structures that contain processed information
///
/// # Arguments
///
/// * 'weapons1': First structure that store information about weapons
/// * 'weapons2': Second structure that store information about weapons
///
pub fn merge_weapons_stats(
    weapons1: &mut HashMap<String, (u32, f64, f64)>,
    weapons2: HashMap<String, (u32, f64, f64)>,
) {
    weapons2.iter().for_each(|(weapon_str_2, stats)| {
        weapons1
            .entry(weapon_str_2.to_string())
            .and_modify(|weapon_1| {
                weapon_1.0 += stats.0;
                weapon_1.1 += stats.1;
                weapon_1.2 += stats.2;
            })
            .or_insert(stats.clone());
    })
}

/// Merge two tuples containing processed information
///
/// # Arguments
///
/// * '(killers1, weapons1)': First tuple containing information about killers and weapons
/// * '(killers2, weapons2)': Second tuple containing information about killers and weapons
///
/// # Returns
///
/// Returns merge killer and weapon information
///
pub fn merge(
    (mut killers1, mut weapons1): (HashMap<String, Player>, HashMap<String, (u32, f64, f64)>),
    (killers2, weapons2): (HashMap<String, Player>, HashMap<String, (u32, f64, f64)>),
) -> (HashMap<String, Player>, HashMap<String, (u32, f64, f64)>) {
    merge_killers(&mut killers1, killers2);
    merge_weapons_stats(&mut weapons1, weapons2);

    (killers1, weapons1)
}

/// Process killer information from a file
///
/// # Arguments
///
/// * 'fields': Line from a file
/// * 'killers': Structure that stores information
///
pub fn process_killer(fields: &Vec<&str>, killers: &mut HashMap<String, Player>) {
    if let Some(killer_name) = fields.get(1) {
        if !killer_name.is_empty() {
            let player = killers
                .entry(killer_name.to_string())
                .or_insert_with(|| Player {
                    deaths: 0,
                    weapons_percentage: HashMap::new(),
                });

            player.deaths += 1;

            if let Some(weapon_name) = fields.get(0) {
                *player
                    .weapons_percentage
                    .entry(weapon_name.to_string())
                    .or_insert(0.0) += 1.0;
            }
        }
    }
}

/// Process weapon information from a file
///
/// # Arguments
///
/// * 'fields': Line from a file
/// * 'weapons': Structure that stores information
///
pub fn process_weapon(fields: &Vec<&str>, weapons: &mut HashMap<String, (u32, f64, f64)>) {
    if let Some(weapon_name) = fields.get(0) {
        if let (
            Some(killer_position_x),
            Some(killer_position_y),
            Some(victim_position_x),
            Some(victim_position_y),
        ) = (
            fields.get(3).and_then(|s| s.trim().parse::<f64>().ok()),
            fields.get(4).and_then(|s| s.trim().parse::<f64>().ok()),
            fields.get(10).and_then(|s| s.trim().parse::<f64>().ok()),
            fields.get(11).and_then(|s| s.trim().parse::<f64>().ok()),
        ) {
            let weapon = weapons
                .entry(weapon_name.to_string())
                .or_insert((0, 0.0, 0.0));

            let distance = ((((killer_position_x - victim_position_x).powi(2)
                + (killer_position_y - victim_position_y).powi(2))
            .sqrt())
                * 100.0)
                .round()
                / 100.0;

            weapon.0 += 1;
            weapon.1 += distance;
            weapon.2 += 1.0;
        } else {
            let weapon = weapons
                .entry(weapon_name.to_string())
                .or_insert((0, 0.0, 0.0));

            weapon.0 += 1;
        }
    }
}

/// Process lines from a file
///
/// # Arguments
///
/// * 'line': Line from a file
/// * 'killers': HashMap that stores the processed information
/// * 'weapons': HashMap that stores the processed information
///
pub fn process_line(
    line: &str,
    killers: &mut HashMap<String, Player>,
    weapons: &mut HashMap<String, (u32, f64, f64)>,
) {
    let fields: Vec<&str> = line.split(",").collect();

    process_killer(&fields, killers);
    process_weapon(&fields, weapons);
}

/// Process entries in a directory
///
/// # Arguments
///
/// * 'entries': Directory entries
///
/// # Returns
///
/// Output set that stores the processed information
pub fn process_directory_entries(entries: std::fs::ReadDir) -> Output {
    let (killers, weapons_stats): (HashMap<String, Player>, HashMap<String, (u32, f64, f64)>) =
        entries
            .flatten()
            .map(|d| d.path())
            .collect::<Vec<PathBuf>>()
            .par_iter()
            .flat_map(|path| {
                let file = File::open(path);
                let reader = BufReader::new(file.unwrap());
                reader.lines().par_bridge()
            })
            .map(|l| {
                let mut killers = HashMap::new();
                let mut weapons = HashMap::new();

                if let Ok(line) = l {
                    process_line(&line, &mut killers, &mut weapons);
                } else {
                    eprintln!("Error: Could not read line");
                }

                (killers, weapons)
            })
            .reduce(|| (HashMap::new(), HashMap::new()), merge);

    let weapons: HashMap<String, Weapon> = process_weapon_stats(weapons_stats);

    Output {
        padron: 107585,
        top_killers: get_top_killers(killers, 10),
        top_weapons: get_top_weapons(weapons, 10),
    }
}

/// Process the information using a given number of threads
///
/// # Arguments
///
/// * 'path': Path to a directory with the files to process
/// * 'num_threads': Number of threads to use for processing
///
/// # Returns
///
/// Output set that stores the processed information
pub fn process_files(path: String, num_threads: usize) -> Output {
    if let Ok(entries) = read_dir(&path) {
        if let Ok(pool) = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
        {
            return pool.install(|| process_directory_entries(entries));
        } else {
            eprintln!("Error: Failed to create global thread pool");
        }
    }

    eprintln!("Error: Failed to read directory");

    Output {
        padron: 107585,
        top_killers: HashMap::new(),
        top_weapons: HashMap::new(),
    }
}

/// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_include;
    use std::time::Instant;

    /// Verify that the time between different threads is better if the number of threads increases
    #[test]
    fn test_process_files_time_comparison() {
        let path = "tests".to_string();

        let instant_1 = Instant::now();
        process_files(path.clone(), 1);
        let time_1 = instant_1.elapsed().as_secs_f64();

        let instant_3 = Instant::now();
        process_files(path.clone(), 3);
        let time_3 = instant_3.elapsed().as_secs_f64();

        assert!(time_1 > time_3);
    }

    /// Verify that the result between different threads is the same
    #[test]
    fn test_process_files_result_comparison() {
        let path = "tests".to_string();

        let output_1 = process_files(path.clone(), 1);
        let output_2 = process_files(path.clone(), 2);
        let output_3 = process_files(path.clone(), 3);
        let output_4 = process_files(path.clone(), 4);

        let json_1 = serde_json::to_value(&output_1).unwrap();
        let json_2 = serde_json::to_value(&output_2).unwrap();
        let json_3 = serde_json::to_value(&output_3).unwrap();
        let json_4 = serde_json::to_value(&output_4).unwrap();

        assert_json_include!(actual: json_1.clone(), expected: json_2);
        assert_json_include!(actual: json_1.clone(), expected: json_3);
        assert_json_include!(actual: json_1.clone(), expected: json_4);
    }

    // Verify that the top of weapons is as expected
    #[test]
    fn test_get_top_weapons() {
        let weapon1 = Weapon {
            deaths_percentage: 45.0,
            average_distance: 100.0,
        };

        let weapon2 = Weapon {
            deaths_percentage: 30.0,
            average_distance: 150.0,
        };

        let weapon3 = Weapon {
            deaths_percentage: 60.0,
            average_distance: 200.0,
        };

        let weapon4 = Weapon {
            deaths_percentage: 25.0,
            average_distance: 250.0,
        };

        let mut weapons = HashMap::new();
        weapons.insert("weapon1".to_string(), weapon1.clone());
        weapons.insert("weapon2".to_string(), weapon2.clone());
        weapons.insert("weapon3".to_string(), weapon3.clone());
        weapons.insert("weapon4".to_string(), weapon4.clone());

        let result = get_top_weapons(weapons, 2);

        let mut expected = HashMap::new();
        expected.insert("weapon3".to_string(), weapon3);
        expected.insert("weapon1".to_string(), weapon1);

        let json_1 = serde_json::to_value(&result).unwrap();
        let json_2 = serde_json::to_value(&expected).unwrap();

        assert_json_include!(actual: json_1.clone(), expected: json_2);
    }

    // Verify that the top of killers is as expected
    #[test]
    fn test_get_top_killers() {
        let mut weapons1 = HashMap::new();
        weapons1.insert("weapons11".to_string(), 30.0);
        weapons1.insert("weapons12".to_string(), 5.0);
        weapons1.insert("weapons13".to_string(), 60.0);
        weapons1.insert("weapons14".to_string(), 5.0);

        let mut weapons2 = HashMap::new();
        weapons2.insert("weapons21".to_string(), 40.0);
        weapons2.insert("weapons22".to_string(), 30.0);
        weapons2.insert("weapons23".to_string(), 10.0);

        let mut weapons3 = HashMap::new();
        weapons3.insert("weapons31".to_string(), 80.0);
        weapons3.insert("weapons32".to_string(), 40.0);

        let player1 = Player {
            deaths: 100,
            weapons_percentage: weapons1,
        };

        let player2 = Player {
            deaths: 80,
            weapons_percentage: weapons2,
        };

        let player3 = Player {
            deaths: 120,
            weapons_percentage: weapons3,
        };

        let mut players = HashMap::new();
        players.insert("player1".to_string(), player1.clone());
        players.insert("player2".to_string(), player2.clone());
        players.insert("player3".to_string(), player3.clone());

        let result = get_top_killers(players, 2);

        let mut expected_weapons1 = HashMap::new();
        expected_weapons1.insert("weapons11".to_string(), 30.0);
        expected_weapons1.insert("weapons12".to_string(), 5.0);
        expected_weapons1.insert("weapons13".to_string(), 60.0);

        let mut expected_weapons3 = HashMap::new();
        expected_weapons3.insert("weapons31".to_string(), 66.67);
        expected_weapons3.insert("weapons32".to_string(), 33.33);

        let expected_player1 = Player {
            deaths: 100,
            weapons_percentage: expected_weapons1,
        };

        let expected_player3 = Player {
            deaths: 120,
            weapons_percentage: expected_weapons3,
        };

        let mut expected = HashMap::new();
        expected.insert("player1".to_string(), expected_player1.clone());
        expected.insert("player3".to_string(), expected_player3.clone());

        let json_1 = serde_json::to_value(&result).unwrap();
        let json_2 = serde_json::to_value(&expected).unwrap();

        assert_json_include!(actual: json_1.clone(), expected: json_2);
    }

    // Verify that the merge between weapons is as expected
    #[test]
    fn test_merge_killers() {
        let mut player1_weapons = HashMap::new();
        player1_weapons.insert("knife".to_string(), 30.0);
        player1_weapons.insert("pistol".to_string(), 70.0);

        let player1 = Player {
            deaths: 100,
            weapons_percentage: player1_weapons,
        };

        let mut player2_weapons = HashMap::new();
        player2_weapons.insert("rifle".to_string(), 80.0);

        let player2 = Player {
            deaths: 80,
            weapons_percentage: player2_weapons,
        };

        let mut player3_weapons = HashMap::new();
        player3_weapons.insert("pistol".to_string(), 50.0);

        let player3 = Player {
            deaths: 50,
            weapons_percentage: player3_weapons,
        };

        let mut killers1 = HashMap::new();
        killers1.insert("player1".to_string(), player1.clone());
        killers1.insert("player2".to_string(), player2.clone());

        let mut killers2 = HashMap::new();
        killers2.insert("player1".to_string(), player1.clone());
        killers2.insert("player3".to_string(), player3.clone());

        merge_killers(&mut killers1, killers2);

        let mut expected_player1_weapons = HashMap::new();
        expected_player1_weapons.insert("knife".to_string(), 60.0);
        expected_player1_weapons.insert("pistol".to_string(), 140.0);

        let expected_player1 = Player {
            deaths: 200,
            weapons_percentage: expected_player1_weapons,
        };

        let mut expected_player2_weapons = HashMap::new();
        expected_player2_weapons.insert("rifle".to_string(), 80.0);

        let expected_player2 = Player {
            deaths: 80,
            weapons_percentage: expected_player2_weapons,
        };

        let mut expected_player3_weapons = HashMap::new();
        expected_player3_weapons.insert("pistol".to_string(), 50.0);

        let expected_player3 = Player {
            deaths: 50,
            weapons_percentage: expected_player3_weapons,
        };

        let mut expected = HashMap::new();
        expected.insert("player1".to_string(), expected_player1.clone());
        expected.insert("player2".to_string(), expected_player2.clone());
        expected.insert("player3".to_string(), expected_player3.clone());

        let json_1 = serde_json::to_value(&killers1).unwrap();
        let json_2 = serde_json::to_value(&expected).unwrap();

        assert_json_include!(actual: json_1.clone(), expected: json_2);
    }

    // Verify that the merge between weapons is as expected
    #[test]
    fn test_merge_weapons() {
        let mut result: HashMap<String, (u32, f64, f64)> = HashMap::new();
        result.insert("weapon1".to_string(), (10, 10.0, 10.0));

        let mut weapon1: HashMap<String, (u32, f64, f64)> = HashMap::new();
        weapon1.insert("weapon1".to_string(), (10, 10.0, 10.0));

        let mut weapon2: HashMap<String, (u32, f64, f64)> = HashMap::new();
        weapon2.insert("weapon2".to_string(), (10, 10.0, 10.0));

        let mut weapon3: HashMap<String, (u32, f64, f64)> = HashMap::new();
        weapon3.insert("weapon3".to_string(), (10, 10.0, 10.0));

        merge_weapons_stats(&mut result, weapon1);
        merge_weapons_stats(&mut result, weapon2);
        merge_weapons_stats(&mut result, weapon3);

        let mut expected = HashMap::new();
        expected.insert("weapon1".to_string(), (20, 20.0, 20.0));
        expected.insert("weapon2".to_string(), (10, 10.0, 10.0));
        expected.insert("weapon3".to_string(), (10, 10.0, 10.0));

        let json_1 = serde_json::to_value(&result).unwrap();
        let json_2 = serde_json::to_value(&expected).unwrap();

        assert_json_include!(actual: json_1.clone(), expected: json_2);
    }
}
