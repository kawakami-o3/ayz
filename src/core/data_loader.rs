use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::master_data::*;

#[derive(Debug)]
pub enum DataLoadError {
    IoError {
        file: String,
        error: std::io::Error,
    },
    ParseError {
        file: String,
        error: ron::error::SpannedError,
    },
    ValidationError(String),
}

impl std::fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataLoadError::IoError { file, error } => {
                write!(f, "Failed to read '{}': {}", file, error)
            }
            DataLoadError::ParseError { file, error } => {
                write!(f, "Failed to parse '{}': {}", file, error)
            }
            DataLoadError::ValidationError(msg) => write!(f, "Data validation error: {}", msg),
        }
    }
}

impl std::error::Error for DataLoadError {}

pub fn load_master_data(data_dir: &Path) -> Result<MasterData, DataLoadError> {
    let items = load_ron::<HashMap<String, ItemDef>>(data_dir, "items.ron")?;
    let equipment = load_ron::<HashMap<String, EquipmentDef>>(data_dir, "equipment.ron")?;
    let monsters = load_ron::<HashMap<String, MonsterStatsDef>>(data_dir, "monsters.ron")?;
    let floors = load_ron::<FloorData>(data_dir, "floors.ron")?;
    let player = load_ron::<PlayerData>(data_dir, "player.ron")?;
    let balance = load_ron::<BalanceData>(data_dir, "balance.ron")?;
    let map = load_ron::<MapData>(data_dir, "map.ron")?;

    let master = MasterData {
        items,
        equipment,
        monsters,
        floors,
        player,
        balance,
        map,
    };
    validate(&master)?;
    Ok(master)
}

fn load_ron<T: serde::de::DeserializeOwned>(
    dir: &Path,
    filename: &str,
) -> Result<T, DataLoadError> {
    let path = dir.join(filename);
    let content = fs::read_to_string(&path).map_err(|e| DataLoadError::IoError {
        file: filename.to_string(),
        error: e,
    })?;
    ron::from_str(&content).map_err(|e| DataLoadError::ParseError {
        file: filename.to_string(),
        error: e,
    })
}

fn validate(data: &MasterData) -> Result<(), DataLoadError> {
    // Validate monster IDs in floor tables
    for entry in &data.floors.monster_table {
        for id in &entry.monsters {
            if !data.monsters.contains_key(id) {
                return Err(DataLoadError::ValidationError(format!(
                    "Monster '{}' referenced in floor table but not defined in monsters.ron",
                    id
                )));
            }
        }
    }

    // Validate item IDs in scroll pool
    for id in &data.floors.scroll_spawns.pool {
        if !data.items.contains_key(id) {
            return Err(DataLoadError::ValidationError(format!(
                "Item '{}' referenced in scroll pool but not defined in items.ron",
                id
            )));
        }
    }

    // Validate item IDs in staff pool
    for id in &data.floors.staff_spawns.pool {
        if !data.items.contains_key(id) {
            return Err(DataLoadError::ValidationError(format!(
                "Item '{}' referenced in staff pool but not defined in items.ron",
                id
            )));
        }
    }

    // Validate equipment IDs in equipment spawns
    for entry in &data.floors.equipment_spawns {
        for id in &entry.weapons {
            if !data.equipment.contains_key(id) {
                return Err(DataLoadError::ValidationError(
                    format!("Equipment '{}' referenced in equipment spawns but not defined in equipment.ron", id)));
            }
        }
        for id in &entry.shields {
            if !data.equipment.contains_key(id) {
                return Err(DataLoadError::ValidationError(
                    format!("Equipment '{}' referenced in equipment spawns but not defined in equipment.ron", id)));
            }
        }
    }

    // Validate food item IDs
    for entry in &data.floors.food_spawns {
        for food_item in &entry.items {
            if !data.items.contains_key(&food_item.id) {
                return Err(DataLoadError::ValidationError(format!(
                    "Item '{}' referenced in food spawns but not defined in items.ron",
                    food_item.id
                )));
            }
        }
    }

    // Validate herb item IDs referenced in herb_spawns
    // (herbs are just count-based, using all items with Herb category)

    Ok(())
}

pub fn resolve_data_dir(cli_arg: Option<&str>) -> PathBuf {
    if let Some(path) = cli_arg {
        return PathBuf::from(path);
    }
    let cwd_data = PathBuf::from("data");
    if cwd_data.is_dir() {
        return cwd_data;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_data = parent.join("data");
            if exe_data.is_dir() {
                return exe_data;
            }
        }
    }
    PathBuf::from("data")
}
