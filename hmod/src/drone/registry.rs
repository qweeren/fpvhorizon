// Registry tracking active drone players across the world.
// Port of DroneRegistry.java.

use std::collections::HashSet;

#[derive(Debug, Default, Clone)]
pub struct DroneRegistry {
    active_drones: HashSet<u128>,
}

impl DroneRegistry {
    pub fn new() -> Self {
        Self {
            active_drones: HashSet::new(),
        }
    }

    pub fn is_drone(&self, id: u128) -> bool {
        self.active_drones.contains(&id)
    }

    pub fn set(&mut self, id: u128, active: bool) {
        if active {
            self.active_drones.insert(id);
        } else {
            self.active_drones.remove(&id);
        }
    }

    pub fn snapshot(&self) -> Vec<u128> {
        self.active_drones.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.active_drones.clear();
    }
}
