// Advanced physics engine for quadcopter aerodynamics, motor thermodynamics, and dynamics.
// Based on AdvancedPhysicsCore.java from Minecraft FPV (GPL-3.0, github.com/minecraft-fpv/mfpv-client).

use super::build::{BuildParam, DroneBuildConfig};
use super::math::{Quat, Vec3f};
use super::physics_settings::{PhysicsConfig, PhysicsParam};
use std::f32::consts::PI;

pub const GRAVITY: f32 = 9.8;
pub const INCHES_PER_METER: f32 = 39.3701;
pub const REF_TEMP: f32 = 293.15;
pub const VAC: f32 = 1.256637e-6;
pub const MAX_TIME_STEP: f32 = 0.0078125;
pub const MIN_TIME_STEP: f32 = 0.001;
pub const PROP_SECTIONS: usize = 5;

#[derive(Debug, Clone)]
pub struct AdvancedPhysicsCore {
    applied_build: Option<DroneBuildConfig>,
    applied_physics: Option<PhysicsConfig>,

    air_density: f32,
    drone_density: f32,
    air_drag_coefficient: f32,
    resistance_per_1000_ft: f32,
    temp_coeff: f32,
    specific_heat: f32,
    heat_transfer: f32,

    drone_mass: f32,
    battery_cells: i32,
    blades: i32,
    prop_radius: f32,
    prop_pitch: f32,
    prop_width: f32,
    motor_width: f32,
    motor_height: f32,
    motor_mass: f32,
    bell_mass: f32,
    blade_mass: f32,
    kv: f32,
    efficiency: f32,
    battery_mah: f32,

    ref_resistance: f32,
    heat_cap: f32,
    motor_surface_area: f32,
    inertia: f32,
    drag_factor: f32,
    max_time_step: f32,
    hover_throttle: f32,
    gravity: f32,
    applied_gravity: f32,

    pub internal_energy: [f32; 4],
    pub temp: [f32; 4],
    pub motor_vel: [f32; 4],
    pub motor_pos: [f32; 4],

    /// Charge drawn from the pack, mAh.
    pub mah_used: f32,
    /// The pack drains and sags (else a full pack that never runs down).
    pub battery_sim: bool,
    /// The motors heat under load (else they stay at the air's temperature).
    pub heat_sim: bool,
    /// The current the pack is giving, amps.
    pub pack_current: f32,

    pub velocity: Vec3f,
    pub drone_up: Vec3f,
    pub drone_left: Vec3f,
}

impl Default for AdvancedPhysicsCore {
    fn default() -> Self {
        let mut core = Self {
            applied_build: None,
            applied_physics: None,
            air_density: 1.225,
            drone_density: 1500.0,
            air_drag_coefficient: 1.05,
            resistance_per_1000_ft: 25.7,
            temp_coeff: 0.00393,
            specific_heat: 400.0,
            heat_transfer: 50.0,
            drone_mass: 0.65,
            battery_cells: 2,
            blades: 3,
            prop_radius: 0.0635,
            prop_pitch: 0.1,
            prop_width: 0.015,
            motor_width: 0.023,
            motor_height: 0.006,
            motor_mass: 0.033,
            bell_mass: 0.012,
            blade_mass: 0.0015,
            kv: 2000.0 * 0.10471976,
            efficiency: 1.0,
            battery_mah: 1400.0,
            ref_resistance: 0.03,
            heat_cap: 13.2,
            motor_surface_area: 0.0016,
            inertia: 2e-5,
            drag_factor: 0.005,
            max_time_step: 0.005,
            hover_throttle: 0.5,
            gravity: GRAVITY,
            applied_gravity: GRAVITY,
            internal_energy: [0.0; 4],
            temp: [REF_TEMP; 4],
            motor_vel: [0.0; 4],
            motor_pos: [0.0; 4],
            mah_used: 0.0,
            battery_sim: true,
            heat_sim: true,
            pack_current: 0.0,
            velocity: Vec3f::ZERO,
            drone_up: Vec3f::UP,
            drone_left: Vec3f::LEFT,
        };
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        core.apply_build_if_changed(&b, &p);
        core.reset_state(0.0);
        core
    }
}

impl AdvancedPhysicsCore {
    pub fn apply_build_if_changed(&mut self, build: &DroneBuildConfig, physics: &PhysicsConfig) {
        // Compared by value: two builds made the same way share a version number.
        if self.applied_build.as_ref() == Some(build)
            && self.applied_physics.as_ref() == Some(physics)
            && self.gravity == self.applied_gravity
        {
            return;
        }
        self.applied_build = Some(*build);
        self.applied_physics = Some(*physics);
        self.applied_gravity = self.gravity;

        self.air_density = physics.get(PhysicsParam::AirDensity);
        self.drone_density = physics.get(PhysicsParam::DroneDensity);
        self.air_drag_coefficient = physics.get(PhysicsParam::DragCoefficient);
        self.resistance_per_1000_ft = physics.get(PhysicsParam::ResistancePer1000Ft);
        self.temp_coeff = physics.get(PhysicsParam::TempCoeff);
        self.specific_heat = physics.get(PhysicsParam::SpecificHeat);
        self.heat_transfer = physics.get(PhysicsParam::HeatTransfer);

        self.drone_mass = build.get(BuildParam::DroneMass);
        self.battery_cells = build.get(BuildParam::BatteryCells).round() as i32;
        self.blades = build.get(BuildParam::Blades).round() as i32;
        self.prop_radius = build.get(BuildParam::PropRadius);
        self.prop_pitch = build.get(BuildParam::PropPitch);
        self.prop_width = build.get(BuildParam::PropWidth);
        self.motor_width = build.get(BuildParam::MotorWidth);
        self.motor_height = build.get(BuildParam::MotorHeight);
        self.motor_mass = build.motor_mass();
        self.bell_mass = build.bell_mass();
        self.blade_mass = build.blade_mass();
        self.kv = build.get(BuildParam::MotorKv) * 0.10471976;
        self.efficiency = build.get(BuildParam::Efficiency);
        self.battery_mah = build.get(BuildParam::BatteryMah);

        let r = self.motor_width * 0.5;
        let poles = Self::get_poles(self.motor_width);
        let slots = Self::get_slots(self.motor_width);
        self.heat_cap = self.specific_heat * self.motor_mass;
        self.motor_surface_area = 2.0 * PI * r * self.motor_height + 2.0 * PI * r * r;

        let mut resistance = self.compute_ref_resistance(r, self.motor_height, self.kv, poles, slots);
        if resistance.is_nan() || resistance.is_infinite() || resistance < 0.001 {
            resistance = 0.03;
        }
        self.ref_resistance = resistance;
        self.inertia = self.compute_motor_inertia();

        let radius = (3.0 * self.drone_mass / (self.drone_density * 4.0 * PI)).cbrt();
        let area = PI * radius * radius;
        self.drag_factor = self.air_density * area * self.air_drag_coefficient / 2.0;

        let time_constant = self.ref_resistance * self.kv * self.kv * self.inertia;
        self.max_time_step = time_constant.clamp(MIN_TIME_STEP, MAX_TIME_STEP);
        self.hover_throttle = self.find_hover_throttle();
    }

    pub fn reset(&mut self, throttle: f32, build: &DroneBuildConfig, physics: &PhysicsConfig) {
        self.apply_build_if_changed(build, physics);
        self.reset_state(throttle);
    }

    pub fn get_hover_throttle(&mut self, build: &DroneBuildConfig, physics: &PhysicsConfig) -> f32 {
        self.apply_build_if_changed(build, physics);
        self.hover_throttle
    }

    pub fn reset_state(&mut self, throttle: f32) {
        for i in 0..4 {
            self.internal_energy[i] = 0.0;
            self.temp[i] = REF_TEMP;
            self.motor_vel[i] = 0.0;
            self.motor_pos[i] = 0.0;
        }
        self.velocity = Vec3f::ZERO;
        self.drone_up = Vec3f::UP;
        self.drone_left = Vec3f::LEFT;

        let vbat = self.get_battery_voltage(throttle.clamp(-1.0, 1.0));
        for motor in 0..4 {
            let use_vbat = (if motor % 2 == 0 { -1.0 } else { 1.0 }) * vbat;
            self.motor_vel[motor] = self.steady_motor_vel(motor, use_vbat, self.temp[motor]);
        }
    }

    fn steady_motor_vel(&self, motor: usize, vbat: f32, temp: f32) -> f32 {
        if vbat == 0.0 {
            return 0.0;
        }
        let direction = vbat.signum();
        let mut lo = 0.0;
        let mut hi = direction * self.kv * self.battery_cells as f32 * 4.2;
        for _ in 0..40 {
            let mid = (lo + hi) * 0.5;
            let accel = self.get_rot_accel(motor, mid, vbat, temp);
            if accel * direction > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        (lo + hi) * 0.5
    }

    pub fn step(&mut self, dt: f32, throttle: f32, orientation: Quat, vel: &mut Vec3f, build: &DroneBuildConfig, physics: &PhysicsConfig) {
        self.apply_build_if_changed(build, physics);

        self.drone_up = orientation.transform_vec3(Vec3f::UP);
        let drone_look = orientation.transform_vec3(Vec3f::new(0.0, 0.0, -1.0));
        self.drone_left = self.drone_up.cross(drone_look);
        self.velocity = *vel;

        // The ESCs give the motors the pack's voltage times the throttle; the pack sags under
        // the current it gave last step.
        let throttle = throttle.clamp(-1.0, 1.0);
        let vbat = self.pack_voltage() * throttle;
        let mut charge = 0.0;
        let gravity_force = Vec3f::new(0.0, -self.gravity * self.drone_mass, 0.0);

        let max_iterations = (dt / self.max_time_step) as usize + 10;
        let mut time_processed = 0.0;

        for _ in 0..max_iterations {
            if dt - time_processed <= 1e-5 {
                break;
            }
            let time_step = (dt - time_processed).min(self.max_time_step);
            let speed = self.velocity.length();
            let mut net_force = self.velocity.normalized_or_zero().mul(-speed * speed * self.drag_factor).add(gravity_force);

            for motor in 0..4 {
                let (thrust_force, _) = self.get_total_forces_on_prop(motor, self.motor_vel[motor]);
                net_force = net_force.add(thrust_force);

                let use_vbat = (if motor % 2 == 0 { -1.0 } else { 1.0 }) * vbat;
                let next_motor_vel = self.advance_motor_vel(motor, self.motor_vel[motor], time_step, use_vbat, self.temp[motor]);
                if self.heat_sim {
                    self.internal_energy[motor] = self.get_next_internal_energy(motor, self.internal_energy[motor], self.temp[motor], use_vbat, self.motor_vel[motor], time_step);
                    self.temp[motor] = self.get_next_temp(self.internal_energy[motor]);
                }
                self.motor_pos[motor] = Self::wrap_angle(self.motor_pos[motor] + self.motor_vel[motor] * time_step);
                self.motor_vel[motor] = next_motor_vel;
            }

            // The pack's current: each ESC draws its motor's current times its duty cycle.
            let current: f32 = (0..4)
                .map(|m| {
                    let use_vbat = (if m % 2 == 0 { -1.0 } else { 1.0 }) * vbat;
                    let i = self.get_current(self.get_hot_resistance(self.temp[m]), use_vbat, self.motor_vel[m]);
                    (i * use_vbat.signum()).max(0.0) * throttle.abs()
                })
                .sum();
            self.pack_current = current;
            charge += current * time_step;

            self.velocity = self.velocity.fma(time_step / self.drone_mass, net_force);
            time_processed += time_step;
        }

        *vel = self.velocity;
        if self.battery_sim {
            self.mah_used += charge * 1000.0 / 3600.0;
        }
    }

    /// Turns the battery's and the motors' heat simulation on or off. Turning heat off cools
    /// the motors at once; turning the battery off fills the pack.
    pub fn set_simulation(&mut self, battery: bool, heat: bool) {
        if !battery && self.battery_sim {
            self.mah_used = 0.0;
        }
        if !heat && self.heat_sim {
            self.temp = [REF_TEMP; 4];
            self.internal_energy = [0.0; 4];
        }
        self.battery_sim = battery;
        self.heat_sim = heat;
    }

    /// A full pack again.
    pub fn new_pack(&mut self) {
        self.mah_used = 0.0;
        self.pack_current = 0.0;
    }

    /// The pack's charge left, 0..1.
    pub fn charge(&self) -> f32 {
        (1.0 - self.mah_used / self.battery_mah.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn battery_capacity(&self) -> f32 {
        self.battery_mah
    }

    pub fn cells(&self) -> i32 {
        self.battery_cells
    }

    /// A LiPo cell's resting voltage at `charge` (0..1) left.
    pub fn cell_open_voltage(charge: f32) -> f32 {
        const CURVE: [(f32, f32); 7] =
            [(0.0, 3.2), (0.05, 3.45), (0.1, 3.55), (0.2, 3.65), (0.5, 3.8), (0.9, 4.05), (1.0, 4.2)];
        let c = charge.clamp(0.0, 1.0);
        for w in CURVE.windows(2) {
            let ((c0, v0), (c1, v1)) = (w[0], w[1]);
            if c <= c1 {
                return v0 + (v1 - v0) * (c - c0) / (c1 - c0);
            }
        }
        4.2
    }

    /// The pack's voltage under load: its cells' resting voltage less the sag of the current
    /// through their internal resistance (about 6 mΩ a cell for a 1300 mAh pack).
    pub fn pack_voltage(&self) -> f32 {
        let cells = self.battery_cells as f32;
        if !self.battery_sim {
            return cells * Self::cell_open_voltage(1.0);
        }
        let cell_resistance = 0.006 * 1300.0 / self.battery_mah.max(50.0);
        let cell = Self::cell_open_voltage(self.charge()) - self.pack_current * cell_resistance;
        cells * cell.max(2.8)
    }

    /// Gravity in m/s² (blocks/s²). Minecraft's own falls at about 32 blocks/s², so real
    /// gravity (9.8) feels floaty there.
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity.clamp(1.0, 50.0);
    }

    /// The throttle that holds altitude with the current build and gravity.
    pub fn hover(&self) -> f32 {
        self.hover_throttle
    }

    /// The motors' mean speed as a fraction of their no-load top speed, 0..1.
    pub fn rpm_fraction(&self) -> f32 {
        let max = self.kv * self.battery_cells as f32 * 4.2;
        if max <= 0.0 {
            return 0.0;
        }
        (self.motor_vel.iter().map(|v| v.abs()).sum::<f32>() / 4.0 / max).clamp(0.0, 1.0)
    }

    pub fn is_overheat(&self) -> bool {
        let mut average_res = 0.0;
        for i in 0..4 {
            average_res += self.get_hot_resistance(self.temp[i]);
        }
        average_res /= 4.0;
        self.ref_resistance / average_res < 0.5
    }

    fn find_hover_throttle(&mut self) -> f32 {
        let weight = self.drone_mass * self.gravity;
        let mut lo = 0.0;
        let mut hi = 1.0;
        for _ in 0..16 {
            let mid = (lo + hi) * 0.5;
            self.reset_state(mid);
            if self.total_thrust() < weight {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        self.reset_state(1.0);
        if self.total_thrust() < weight {
            return 1.0;
        }
        (lo + hi) * 0.5
    }

    /// Thrust along the drone's up at the motors' current speed, newtons.
    pub fn static_thrust(&self) -> f32 {
        self.total_thrust()
    }

    fn total_thrust(&self) -> f32 {
        let mut thrust = 0.0;
        for motor in 0..4 {
            let (force, _) = self.get_total_forces_on_prop(motor, self.motor_vel[motor]);
            thrust += force.dot(self.drone_up);
        }
        thrust
    }

    fn get_motor_to_prop(rotation_angle: f32, world_drone_up: Vec3f, world_drone_right: Vec3f) -> Vec3f {
        Quat::from_axis_angle(world_drone_up, rotation_angle).transform_vec3(world_drone_right)
    }

    fn get_world_prop_velocity(world_drone_up: Vec3f, world_drone_velocity: Vec3f, motor_to_prop: Vec3f, angular_speed: f32, section_radius: f32) -> Vec3f {
        let prop_velocity = world_drone_up.cross(motor_to_prop).normalized_or_zero();
        prop_velocity.mul(angular_speed * section_radius).add(world_drone_velocity)
    }

    fn get_free_vel_rel_prop_planar(world_prop_velocity: Vec3f, world_velocity_free: Vec3f, motor_to_prop: Vec3f) -> Vec3f {
        let free_vel_rel_prop = world_velocity_free.sub(world_prop_velocity);
        let normal_proj = free_vel_rel_prop.project(motor_to_prop);
        free_vel_rel_prop.sub(normal_proj)
    }

    fn get_aoa(motor_number: usize, prop_pitch: f32, section_radius: f32, free_vel_rel_prop_planar: Vec3f, world_drone_up: Vec3f, motor_to_prop: Vec3f) -> f32 {
        let pitch_angle = prop_pitch.atan2(2.0 * PI * section_radius);
        let wind_direction = free_vel_rel_prop_planar.mul(-1.0).normalized_or_zero();
        let mut forward_spin_direction = world_drone_up.cross(motor_to_prop);
        if motor_number % 2 == 0 {
            forward_spin_direction = forward_spin_direction.mul(-1.0);
        }
        let mut sign = world_drone_up.dot(wind_direction).signum();
        if sign == 0.0 {
            sign = 1.0;
        }
        let wind_angle = sign * forward_spin_direction.angle_between(wind_direction);
        let mut angle_of_attack = pitch_angle - wind_angle;
        if motor_number % 2 == 0 {
            angle_of_attack *= -1.0;
        }
        angle_of_attack
    }

    fn get_prop_drag_coefficient(aoa: f32) -> f32 {
        let min: f32 = 0.4;
        let max: f32 = 1.8;
        let min_wrapped = min.powf(1.15);
        let max_wrapped = max.powf(1.15);
        let general_shape = -(aoa * 2.0).cos() + 1.0;
        let scaled_and_offset = general_shape / 2.0 * (max_wrapped - min_wrapped) + min_wrapped;
        scaled_and_offset.powf(0.8695652)
    }

    fn get_prop_lift_coefficient(mut aoa: f32) -> f32 {
        aoa = if aoa < 0.0 {
            PI + (aoa % PI)
        } else {
            aoa % PI
        };
        let alp = 7.5 * (PI / 180.0);
        let sqrt2_over_pi = (2.0 / PI).sqrt();
        let mut naca = sqrt2_over_pi * aoa * aoa * (-aoa * aoa / (2.0 * alp * alp)).exp() / (alp * alp * alp);
        let antiaoa = PI - aoa;
        let mut antinaca = -sqrt2_over_pi * antiaoa * antiaoa * (-antiaoa * antiaoa / (2.0 * alp * alp)).exp() / (alp * alp * alp);
        let plate = (aoa * 2.0).sin();
        naca /= 7.0;
        antinaca /= 7.0;
        plate + naca + antinaca
    }

    fn get_prop_drag_force(&self, free_vel_rel_prop_planar: Vec3f, drag_coefficient: f32, aoa: f32, section_length: f32, section_width: f32) -> Vec3f {
        let flow_speed = free_vel_rel_prop_planar.length();
        let thickness = 0.002;
        let reference_area = (section_length * section_width * aoa.sin()).abs() + (section_length * thickness * aoa.cos()).abs();
        let force_mag = drag_coefficient * self.air_density * flow_speed * flow_speed * reference_area / 2.0;
        free_vel_rel_prop_planar.normalized_or_zero().mul(force_mag)
    }

    fn get_prop_lift_force(&self, free_vel_rel_prop_planar: Vec3f, motor_to_prop: Vec3f, lift_coefficient: f32, section_length: f32, section_width: f32) -> Vec3f {
        let flow_speed = free_vel_rel_prop_planar.length();
        let reference_area = section_length * section_width;
        let force_mag = lift_coefficient * self.air_density * flow_speed * flow_speed * reference_area / 2.0;
        let wind_direction = free_vel_rel_prop_planar.mul(-1.0).normalized_or_zero();
        motor_to_prop.normalized_or_zero().cross(wind_direction).mul(force_mag)
    }

    fn get_total_forces_on_prop(&self, motor_number: usize, rotvel: f32) -> (Vec3f, Vec3f) {
        let mut net_force = Vec3f::ZERO;
        let mut net_drag_torque = Vec3f::ZERO;
        let world_velocity_free = Vec3f::ZERO;
        let drone_right = self.drone_left.mul(-1.0);
        let init_rotation_angle = self.motor_pos[motor_number];
        let section_length = self.prop_radius / PROP_SECTIONS as f32;

        for i in 0..self.blades {
            let rotation_angle = init_rotation_angle + 2.0 * PI * (i as f32) / (self.blades as f32);
            let motor_to_prop = Self::get_motor_to_prop(rotation_angle, self.drone_up, drone_right);

            for j in 0..PROP_SECTIONS {
                let section_radius = ((j as f32) + 0.5) / (PROP_SECTIONS as f32) * self.prop_radius;
                if section_radius < self.motor_width * 0.5 {
                    continue;
                }
                let mut section_width = self.prop_width;
                if section_radius < self.motor_width {
                    section_width *= section_radius / self.motor_width;
                }

                let world_prop_velocity = Self::get_world_prop_velocity(self.drone_up, self.velocity, motor_to_prop, rotvel, section_radius);
                let free_vel_rel_prop_planar = Self::get_free_vel_rel_prop_planar(world_prop_velocity, world_velocity_free, motor_to_prop);
                let aoa = Self::get_aoa(motor_number, self.prop_pitch, section_radius, free_vel_rel_prop_planar, self.drone_up, motor_to_prop);
                let drag_coeff = Self::get_prop_drag_coefficient(aoa);
                let lift_coeff = Self::get_prop_lift_coefficient(aoa);

                let drag_force = self.get_prop_drag_force(free_vel_rel_prop_planar, drag_coeff, aoa, section_length, section_width);
                let lift_force = self.get_prop_lift_force(free_vel_rel_prop_planar, motor_to_prop, lift_coeff, section_length, section_width).mul(self.efficiency);

                net_drag_torque = net_drag_torque.add(motor_to_prop.cross(drag_force).mul(section_radius));
                net_force = net_force.add(drag_force).add(lift_force);
            }
        }

        (net_force, net_drag_torque)
    }

    fn dflux_dt(x: f32, slot_w: f32, r: f32, h: f32, m: f32, rotvel: f32) -> f32 {
        let gap = 0.00159;
        let xlow = -slot_w / 2.0;
        let xhigh = slot_w / 2.0;
        let gmin = (gap * gap + (x + xlow) * (x + xlow)) as f64;
        let gmax = (gap * gap + (x + xhigh) * (x + xhigh)) as f64;
        let c1 = (VAC * h * r * rotvel * m / (4.0 * PI)) as f64;
        let c2 = (3.0 * gap * gap) as f64;
        (c1 * (c2 * (gmax.powf(-2.5) - gmin.powf(-2.5)) - (gmax.powf(-1.5) - gmin.powf(-1.5)))) as f32
    }

    fn get_poles(motor_width: f32) -> f32 {
        if motor_width >= 0.018 { 14.0 } else { 12.0 }
    }

    fn get_slots(motor_width: f32) -> f32 {
        if motor_width >= 0.018 { 12.0 } else { 9.0 }
    }

    fn get_slot_w(r: f32, slots: f32) -> f32 {
        2.0 * PI * r / slots - 0.0005
    }

    fn get_mag_w(r: f32, poles: f32) -> f32 {
        2.0 * PI * r / poles - 0.0005
    }

    fn get_pole_magnetic_moment(r: f32, h: f32, poles: f32) -> f32 {
        let mag_w = Self::get_mag_w(r, poles);
        let mag_d = 0.001;
        1177746.6 * mag_w * h * mag_d
    }

    fn get_n(&self, r: f32, h: f32, kv: f32, poles: f32, slots: f32) -> f32 {
        let slot_w = Self::get_slot_w(r, slots);
        let mag_w = Self::get_mag_w(r, poles);
        let m = Self::get_pole_magnetic_moment(r, h, poles);
        let x1 = -mag_w + mag_w / 2.0;
        let x2 = mag_w / 2.0;
        let x3 = mag_w + mag_w / 2.0;
        let d1 = Self::dflux_dt(x1, slot_w, r, h, m, 1.0);
        let d2 = Self::dflux_dt(x2, slot_w, r, h, -m, 1.0);
        let d3 = Self::dflux_dt(x3, slot_w, r, h, m, 1.0);
        let d = d1 + d2 + d3;
        (1.0 / (d * kv)).round()
    }

    fn compute_ref_resistance(&self, r: f32, h: f32, kv: f32, poles: f32, slots: f32) -> f32 {
        let slot_w = Self::get_slot_w(r, slots);
        let n = self.get_n(r, h, kv, poles, slots);
        let loop_peri = 2.0 * h + 2.0 * slot_w;
        let wire_length_inches = n * loop_peri * INCHES_PER_METER;
        wire_length_inches / 12.0 / 1000.0 * self.resistance_per_1000_ft
    }

    fn compute_motor_inertia(&self) -> f32 {
        let blade_length = self.prop_radius;
        let prop_inertia = (self.blades as f32) / 12.0 * self.blade_mass * (4.0 * blade_length * blade_length + self.prop_width * self.prop_width);
        let r = self.motor_width * 0.5;
        let bell_inertia = self.bell_mass * r * r;
        prop_inertia + bell_inertia
    }

    fn get_hot_resistance(&self, temp: f32) -> f32 {
        self.ref_resistance * (1.0 + self.temp_coeff * (temp - REF_TEMP))
    }

    fn get_current(&self, resistance: f32, vbat: f32, rotvel: f32) -> f32 {
        let back_emf = -rotvel / self.kv;
        (vbat + back_emf) / resistance
    }

    fn get_torque(&self, current: f32) -> f32 {
        current / self.kv
    }

    fn get_rot_accel(&self, motor: usize, rotvel: f32, vbat: f32, temp: f32) -> f32 {
        let resistance = self.get_hot_resistance(temp);
        let current = self.get_current(resistance, vbat, rotvel);
        let torque = self.get_torque(current);
        let (_, drag_torque_vec) = self.get_total_forces_on_prop(motor, rotvel);
        let drag_torque = drag_torque_vec.dot(self.drone_up);
        (torque + drag_torque) / self.inertia
    }

    fn advance_motor_vel(&self, motor: usize, rotvel: f32, time_step: f32, vbat: f32, temp: f32) -> f32 {
        let k1 = self.get_rot_accel(motor, rotvel, vbat, temp);
        let k2 = self.get_rot_accel(motor, rotvel + time_step / 2.0 * k1, vbat, temp);
        let k3 = self.get_rot_accel(motor, rotvel + time_step / 2.0 * k2, vbat, temp);
        let k4 = self.get_rot_accel(motor, rotvel + time_step * k3, vbat, temp);

        let mut next_vel = rotvel + time_step * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
        if next_vel.is_infinite() || next_vel.is_nan() {
            next_vel = 0.0;
        }
        let max_speed = self.kv * self.battery_cells as f32 * 4.2;
        if next_vel.abs() > max_speed {
            next_vel = max_speed * next_vel.signum();
        }
        next_vel
    }

    fn get_battery_voltage(&self, throt: f32) -> f32 {
        self.pack_voltage() * throt
    }

    fn get_internal_energy_lost(&self, temp: f32, time_step: f32) -> f32 {
        let time_constant = self.heat_cap / (self.heat_transfer * self.motor_surface_area);
        let cooled_temp = REF_TEMP + (temp - REF_TEMP) * (-time_step / time_constant).exp();
        self.heat_cap * (cooled_temp - temp)
    }

    fn get_internal_energy_gained(&self, vbat: f32, rotvel: f32, time_step: f32) -> f32 {
        let current = self.get_current(self.ref_resistance, vbat, rotvel);
        let heating_power = current * current * self.ref_resistance;
        heating_power * time_step
    }

    fn get_next_internal_energy(&mut self, motor: usize, internal_energy: f32, temp_in: f32, vbat: f32, rotvel: f32, time_step: f32) -> f32 {
        let mut next_energy = internal_energy + self.get_internal_energy_lost(temp_in, time_step) + self.get_internal_energy_gained(vbat, rotvel, time_step);
        if next_energy.is_infinite() || next_energy.is_nan() {
            next_energy = 0.0;
            self.temp[motor] = 0.0;
        }
        next_energy
    }

    fn get_next_temp(&self, next_internal_energy: f32) -> f32 {
        next_internal_energy / self.heat_cap + REF_TEMP
    }

    fn wrap_angle(angle: f32) -> f32 {
        let two_pi = 2.0 * PI;
        let wrapped = angle % two_pi;
        if wrapped < 0.0 { wrapped + two_pi } else { wrapped }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_initialization_and_hover() {
        let mut core = AdvancedPhysicsCore::default();
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let hover = core.get_hover_throttle(&b, &p);
        assert!(hover > 0.1 && hover < 0.9, "Hover throttle expected in sensible range, got {}", hover);
    }

    #[test]
    fn default_build_is_a_racer() {
        // A 5" racer hovers around a quarter throttle with a thrust-to-weight near 10.
        let mut core = AdvancedPhysicsCore::default();
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let hover = core.get_hover_throttle(&b, &p);
        assert!(hover > 0.15 && hover < 0.4, "hover {hover}");
        core.reset(1.0, &b, &p);
        let twr = core.total_thrust() / (b.get(BuildParam::DroneMass) * GRAVITY);
        assert!(twr > 6.0, "thrust-to-weight {twr}");
    }

    #[test]
    fn more_gravity_needs_more_hover_throttle() {
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let mut core = AdvancedPhysicsCore::default();
        let real = core.get_hover_throttle(&b, &p);
        core.set_gravity(20.0);
        let heavy = core.get_hover_throttle(&b, &p);
        assert!(heavy > real, "{heavy} should exceed {real}");
    }

    #[test]
    fn a_new_build_takes_even_with_the_same_version() {
        let mut core = AdvancedPhysicsCore::default();
        let p = PhysicsConfig::default();
        let mut six = DroneBuildConfig::default();
        six.set(BuildParam::BatteryCells, 6.0);
        let mut four = DroneBuildConfig::default();
        four.set(BuildParam::BatteryCells, 4.0);
        assert_eq!(six.version, four.version);
        core.reset(0.0, &six, &p);
        assert_eq!(core.cells(), 6);
        core.reset(0.0, &four, &p);
        assert_eq!(core.cells(), 4);
    }

    #[test]
    fn the_pack_drains_and_sags() {
        let mut core = AdvancedPhysicsCore::default();
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let hover = core.get_hover_throttle(&b, &p);
        core.reset(hover, &b, &p);
        let rested = core.pack_voltage();
        let mut vel = Vec3f::ZERO;
        for _ in 0..20 * 60 {
            core.step(0.05, 1.0, Quat::IDENTITY, &mut vel, &b, &p);
            vel = Vec3f::ZERO;
        }
        eprintln!("1 min full throttle: {:.0} mAh, {:.1} A, {:.2} V (rested {:.2} V)", core.mah_used, core.pack_current, core.pack_voltage(), rested);
        assert!(core.mah_used > 100.0, "a minute at full throttle uses real charge");
        assert!(core.pack_voltage() < rested, "the pack sags and drains");
    }

    #[test]
    fn battery_and_heat_can_be_switched_off() {
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let mut core = AdvancedPhysicsCore::default();
        core.reset(0.0, &b, &p);
        core.set_simulation(false, false);
        let full = core.pack_voltage();
        let mut vel = Vec3f::ZERO;
        for _ in 0..1200 {
            core.step(0.05, 1.0, Quat::IDENTITY, &mut vel, &b, &p);
        }
        assert_eq!(core.mah_used, 0.0, "no drain");
        assert_eq!(core.pack_voltage(), full, "no sag");
        assert!(core.temp.iter().all(|t| *t == REF_TEMP), "no heat");
        assert!(!core.is_overheat());
        core.set_simulation(true, true);
        for _ in 0..200 {
            core.step(0.05, 1.0, Quat::IDENTITY, &mut vel, &b, &p);
        }
        assert!(core.mah_used > 0.0 && core.pack_voltage() < full, "back on, it drains and sags");
        assert!(core.temp.iter().all(|t| *t > REF_TEMP), "back on, it heats");
    }

    #[test]
    fn test_physics_step_produces_motion() {
        let mut core = AdvancedPhysicsCore::default();
        let b = DroneBuildConfig::default();
        let p = PhysicsConfig::default();
        let mut vel = Vec3f::ZERO;
        let orient = Quat::IDENTITY;
        // Step with full throttle upwards
        for _ in 0..10 {
            core.step(0.02, 1.0, orient, &mut vel, &b, &p);
        }
        assert!(vel.y > 0.0, "Drone should accelerate upward under full throttle, got vel.y = {}", vel.y);
    }
}


