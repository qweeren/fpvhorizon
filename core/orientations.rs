// Slerp-smoothed orientation tracking for remote drone players.
// Port of DroneOrientations.java.

use super::math::Quat;
use std::collections::HashMap;

const SMOOTHING_SECONDS: f32 = 0.04;

#[derive(Debug, Clone)]
struct OrientationEntry {
    target: Quat,
    display: Quat,
    last_nanos: u64,
}

#[derive(Debug, Default, Clone)]
pub struct DroneOrientations {
    entries: HashMap<u128, OrientationEntry>,
}

impl DroneOrientations {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn set(&mut self, id: u128, mut q: Quat, now_nanos: u64) {
        if q.length_squared() < 1e-6 || !q.x.is_finite() || !q.y.is_finite() || !q.z.is_finite() || !q.w.is_finite() {
            return;
        }
        q = q.normalize();

        if let Some(entry) = self.entries.get_mut(&id) {
            entry.target = q;
        } else {
            self.entries.insert(id, OrientationEntry {
                target: q,
                display: q,
                last_nanos: now_nanos,
            });
        }
    }

    pub fn has(&self, id: u128) -> bool {
        self.entries.contains_key(&id)
    }

    pub fn get(&mut self, id: u128, now_nanos: u64) -> Option<Quat> {
        let entry = self.entries.get_mut(&id)?;
        let dt = (now_nanos.saturating_sub(entry.last_nanos) as f32) * 1e-9;
        entry.last_nanos = now_nanos;

        let alpha = 1.0 - (-dt / SMOOTHING_SECONDS).exp();
        entry.display = entry.display.slerp(entry.target, alpha.clamp(0.0, 1.0));
        Some(entry.display)
    }

    pub fn remove(&mut self, id: u128) {
        self.entries.remove(&id);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slerp_interpolation() {
        let mut orientations = DroneOrientations::new();
        let q1 = Quat::IDENTITY;
        let q2 = Quat::from_axis_angle(super::super::math::Vec3f::UP, std::f32::consts::PI / 2.0);

        orientations.set(123, q1, 1000);
        orientations.set(123, q2, 1000);

        let interpolated = orientations.get(123, 1000 + 40_000_000).unwrap();
        assert!(interpolated.y > 0.0);
    }
}
