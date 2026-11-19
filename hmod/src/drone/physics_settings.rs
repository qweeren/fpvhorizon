// Physical environment and motor thermal coefficients.
// Direct port of PhysicsSettings.java.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsParam {
    AirDensity,
    DroneDensity,
    DragCoefficient,
    ResistancePer1000Ft,
    TempCoeff,
    SpecificHeat,
    HeatTransfer,
}

impl PhysicsParam {
    pub const ALL: [PhysicsParam; 7] = [
        PhysicsParam::AirDensity,
        PhysicsParam::DroneDensity,
        PhysicsParam::DragCoefficient,
        PhysicsParam::ResistancePer1000Ft,
        PhysicsParam::TempCoeff,
        PhysicsParam::SpecificHeat,
        PhysicsParam::HeatTransfer,
    ];

    pub fn key(self) -> &'static str {
        match self {
            PhysicsParam::AirDensity => "air_density",
            PhysicsParam::DroneDensity => "drone_density",
            PhysicsParam::DragCoefficient => "drag_coefficient",
            PhysicsParam::ResistancePer1000Ft => "resistance_per_1000_ft",
            PhysicsParam::TempCoeff => "temp_coeff",
            PhysicsParam::SpecificHeat => "specific_heat",
            PhysicsParam::HeatTransfer => "heat_transfer",
        }
    }

    pub fn default_value(self) -> f32 {
        match self {
            PhysicsParam::AirDensity => 1.225,
            PhysicsParam::DroneDensity => 1500.0,
            PhysicsParam::DragCoefficient => 1.05,
            PhysicsParam::ResistancePer1000Ft => 25.7,
            PhysicsParam::TempCoeff => 0.00393,
            PhysicsParam::SpecificHeat => 400.0,
            PhysicsParam::HeatTransfer => 50.0,
        }
    }

    pub fn min(self) -> f32 {
        match self {
            PhysicsParam::AirDensity => 0.1,
            PhysicsParam::DroneDensity => 200.0,
            PhysicsParam::DragCoefficient => 0.1,
            PhysicsParam::ResistancePer1000Ft => 1.0,
            PhysicsParam::TempCoeff => 0.0,
            PhysicsParam::SpecificHeat => 100.0,
            PhysicsParam::HeatTransfer => 1.0,
        }
    }

    pub fn max(self) -> f32 {
        match self {
            PhysicsParam::AirDensity => 3.0,
            PhysicsParam::DroneDensity => 5000.0,
            PhysicsParam::DragCoefficient => 3.0,
            PhysicsParam::ResistancePer1000Ft => 100.0,
            PhysicsParam::TempCoeff => 0.01,
            PhysicsParam::SpecificHeat => 1000.0,
            PhysicsParam::HeatTransfer => 500.0,
        }
    }

    pub fn step(self) -> f32 {
        match self {
            PhysicsParam::AirDensity => 0.005,
            PhysicsParam::DroneDensity => 50.0,
            PhysicsParam::DragCoefficient => 0.01,
            PhysicsParam::ResistancePer1000Ft => 0.1,
            PhysicsParam::TempCoeff => 0.00001,
            PhysicsParam::SpecificHeat => 5.0,
            PhysicsParam::HeatTransfer => 1.0,
        }
    }

    pub fn snap(self, val: f32) -> f32 {
        let min = self.min();
        let max = self.max();
        let step = self.step();
        let steps = ((val - min) / step).round();
        (min + steps * step).clamp(min, max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsConfig {
    pub values: [f32; 7],
    pub version: u32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        let mut vals = [0.0; 7];
        for (i, p) in PhysicsParam::ALL.iter().enumerate() {
            vals[i] = p.default_value();
        }
        Self { values: vals, version: 1 }
    }
}

impl PhysicsConfig {
    pub fn get(&self, param: PhysicsParam) -> f32 {
        self.values[param as usize]
    }

    pub fn set(&mut self, param: PhysicsParam, value: f32) {
        self.values[param as usize] = param.snap(value);
        self.version = self.version.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
        self.version = self.version.wrapping_add(1);
    }

    pub fn parse_properties(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim();
                for param in PhysicsParam::ALL {
                    if param.key() == k {
                        if let Ok(num) = v.parse::<f32>() {
                            self.set(param, num);
                        }
                    }
                }
            }
        }
    }

    pub fn to_properties(&self) -> String {
        let mut out = String::from("# FPV Horizon physics constants, in SI units.\n");
        for param in PhysicsParam::ALL {
            out.push_str(&format!("{}={}\n", param.key(), self.get(param)));
        }
        out
    }
}
