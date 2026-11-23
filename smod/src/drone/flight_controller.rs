// The flight controller: a rate PID loop per axis and a quad-X motor mixer, as Betaflight
// flies a quad. FPV Horizon's own code; the gain scales are Betaflight's (GPL-3.0), so P, I
// and D read as they do in Betaflight and Liftoff.

/// Betaflight's term scales: a gain of 1 gives this much PID sum per deg/s of error (P),
/// per deg of error integrated (I) and per deg/s² of the gyro's change (D).
pub const P_SCALE: f32 = 0.032029;
pub const I_SCALE: f32 = 0.244381;
pub const D_SCALE: f32 = 0.000529;
/// The PID sum is in thousandths of the motors' range.
pub const MIXER_SCALE: f32 = 1000.0;
/// The most the PID sum may ask of the motors: roll and pitch, then yaw.
pub const PIDSUM_LIMIT: [f32; 3] = [500.0, 500.0, 400.0];
/// The most the I term may hold.
pub const ITERM_LIMIT: f32 = 400.0;
/// The setpoint's smoothing for the D term, Hz: a stick snapped in one frame becomes a
/// push over a few loops instead of one spike.
pub const SETPOINT_SMOOTHING: f32 = 20.0;
/// The throttle that turns on air mode's I term after arming, so it doesn't wind up on the ground.
pub const AIRMODE_ACTIVATE: f32 = 0.25;

/// The roll and pitch authority (rad/s² per unit of differential motor command at a hover)
/// the default gains are tuned on: a 5" freestyle quad.
pub const REFERENCE_AUTHORITY: [f32; 2] = [2350.0, 1630.0];

/// How much a build's roll and pitch PID output is scaled so the default gains suit it, as each
/// drone ships with its own tune: half way (in ratio) to the 5"'s loop gain, so a whoop still
/// feels twitchier than a 7".
pub fn authority_scale(authority: [f32; 2]) -> [f32; 2] {
    std::array::from_fn(|i| (REFERENCE_AUTHORITY[i] / authority[i].max(1.0)).sqrt().clamp(0.3, 3.0))
}

/// Each motor's share of roll, pitch and yaw, quad X, motors in [`super::physics::spin`]'s
/// order: front right, back right, back left, front left. Right roll lifts the left side, nose
/// down lifts the back, and right yaw speeds the counter-clockwise props, whose drag turns the
/// frame clockwise.
pub const QUAD_X: [[f32; 3]; 4] = [
    [-1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PidConfig {
    /// Gains per axis: roll, pitch, yaw.
    pub p: [f32; 3],
    pub i: [f32; 3],
    pub d: [f32; 3],
    /// How much of the stick's own movement the D term answers (Betaflight's D setpoint
    /// weight): more makes the drone start and stop its turns sharper.
    pub d_setpoint_weight: f32,
    /// Throttle PID attenuation: P and D fall by `tpa_rate` from `tpa_breakpoint` to full
    /// throttle, where the props bite harder.
    pub tpa_rate: f32,
    pub tpa_breakpoint: f32,
    /// I-term relax: the I term rests while the stick moves faster than this (deg/s, after a
    /// 15 Hz high-pass), so flips and rolls don't bounce back.
    pub iterm_relax_threshold: f32,
    /// The D term's low-pass, Hz.
    pub dterm_cutoff: f32,
}

impl Default for PidConfig {
    fn default() -> Self {
        Self {
            p: [30.0, 32.0, 30.0],
            i: [50.0, 50.0, 50.0],
            d: [20.0, 22.0, 0.0],
            d_setpoint_weight: 0.6,
            tpa_rate: 0.3,
            tpa_breakpoint: 0.5,
            iterm_relax_threshold: 40.0,
            dterm_cutoff: 100.0,
        }
    }
}

/// The rate loop's memory.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RateController {
    iterm: [f32; 3],
    prev_gyro: [f32; 3],
    /// The setpoint smoothed for the D term's share of it (as a radio link's RC smoothing).
    setpoint_smooth: [f32; 3],
    prev_setpoint_smooth: [f32; 3],
    dterm: [f32; 3],
    setpoint_lpf: [f32; 3],
    primed: bool,
    /// Air mode is on: the throttle went past [`AIRMODE_ACTIVATE`] since arming.
    pub airmode_active: bool,
}

/// A first-order low-pass's blend factor for a step of `dt` at `cutoff` Hz.
fn pt1(cutoff: f32, dt: f32) -> f32 {
    1.0 - (-dt * 2.0 * std::f32::consts::PI * cutoff).exp()
}

impl RateController {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// One loop: the gyro (deg/s, roll pitch yaw as the sticks turn them) against the setpoint
    /// (deg/s), at `throttle` (0..1). Returns the PID sums, as shares of the motors' range.
    pub fn update(&mut self, cfg: &PidConfig, setpoint: [f32; 3], gyro: [f32; 3], throttle: f32, dt: f32) -> [f32; 3] {
        if dt <= 0.0 {
            return [0.0; 3];
        }
        if !self.primed {
            self.prev_gyro = gyro;
            self.setpoint_smooth = setpoint;
            self.prev_setpoint_smooth = setpoint;
            self.setpoint_lpf = setpoint;
            self.primed = true;
        }
        if throttle >= AIRMODE_ACTIVATE {
            self.airmode_active = true;
        }
        let tpa = if throttle > cfg.tpa_breakpoint {
            1.0 - cfg.tpa_rate * ((throttle - cfg.tpa_breakpoint) / (1.0 - cfg.tpa_breakpoint).max(0.01)).min(1.0)
        } else {
            1.0
        };
        let d_lpf = pt1(cfg.dterm_cutoff, dt);
        let relax_lpf = pt1(15.0, dt);
        let smooth_lpf = pt1(SETPOINT_SMOOTHING, dt);

        let mut out = [0.0; 3];
        for axis in 0..3 {
            let error = setpoint[axis] - gyro[axis];
            let p = cfg.p[axis] * P_SCALE * error * tpa;

            self.setpoint_lpf[axis] += (setpoint[axis] - self.setpoint_lpf[axis]) * relax_lpf;
            let moving = (setpoint[axis] - self.setpoint_lpf[axis]).abs();
            let relax = (1.0 - moving / cfg.iterm_relax_threshold.max(1.0)).max(0.0);
            if self.airmode_active {
                self.iterm[axis] += cfg.i[axis] * I_SCALE * error * relax * dt;
                self.iterm[axis] = self.iterm[axis].clamp(-ITERM_LIMIT, ITERM_LIMIT);
            } else {
                self.iterm[axis] = 0.0;
            }

            // D answers the gyro's change, plus a share of the stick's, through a low-pass.
            let d_gyro = (gyro[axis] - self.prev_gyro[axis]) / dt;
            self.setpoint_smooth[axis] += (setpoint[axis] - self.setpoint_smooth[axis]) * smooth_lpf;
            let d_setpoint = (self.setpoint_smooth[axis] - self.prev_setpoint_smooth[axis]) / dt;
            self.prev_setpoint_smooth[axis] = self.setpoint_smooth[axis];
            let raw = cfg.d_setpoint_weight * d_setpoint - d_gyro;
            self.dterm[axis] += (raw - self.dterm[axis]) * d_lpf;
            let d = cfg.d[axis] * D_SCALE * self.dterm[axis] * tpa;
            self.prev_gyro[axis] = gyro[axis];

            let limit = PIDSUM_LIMIT[axis];
            out[axis] = (p + self.iterm[axis] + d).clamp(-limit, limit) / MIXER_SCALE;
        }
        out
    }

    /// Lets the I term bleed away, while the drone sits on something.
    pub fn settle(&mut self, factor: f32) {
        for i in self.iterm.iter_mut() {
            *i *= factor;
        }
    }

    pub fn iterm(&self) -> [f32; 3] {
        self.iterm.map(|i| i / MIXER_SCALE)
    }
}

/// The motor mix, air mode: each motor's command (0..1, or -1..1 for 3D) for `throttle` and
/// the PID sums `pid` (roll, pitch, yaw). The PID always gets the room it needs, the throttle
/// moving to make it (scaled down only when the corrections alone span more than the motors'
/// range), so the drone stays in control at zero throttle and at full.
pub fn mix(throttle: f32, pid: [f32; 3], three_d: bool) -> [f32; 4] {
    let mut m = QUAD_X.map(|c| c[0] * pid[0] + c[1] * pid[1] + c[2] * pid[2]);
    let lo = m.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = m.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = hi - lo;
    let (mut lo, mut hi) = (lo, hi);
    if range > 1.0 {
        for v in m.iter_mut() {
            *v /= range;
        }
        lo /= range;
        hi /= range;
    }
    // The span the motors run in: 0..1, or the reversed half in 3D with the stick low.
    let (bottom, top) = if three_d && throttle < 0.0 { (-1.0, 0.0) } else { (0.0, 1.0) };
    // The corrections now span at most the motors' range, so this window is empty only by
    // rounding.
    let (from, to) = (bottom - lo, top - hi);
    let t = if from <= to { throttle.clamp(from, to) } else { (from + to) * 0.5 };
    m.map(|v| (t + v).clamp(bottom, top))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixer_turns_each_axis_with_opposite_corners() {
        // Right roll: the left motors (2, 3) up, the right ones down.
        let m = mix(0.5, [0.1, 0.0, 0.0], false);
        assert!(m[2] > 0.5 && m[3] > 0.5 && m[0] < 0.5 && m[1] < 0.5, "{m:?}");
        // Nose down: the back (1, 2) up.
        let m = mix(0.5, [0.0, 0.1, 0.0], false);
        assert!(m[1] > 0.5 && m[2] > 0.5 && m[0] < 0.5 && m[3] < 0.5, "{m:?}");
        // Right yaw: the counter-clockwise props (1, 3) up.
        let m = mix(0.5, [0.0, 0.0, 0.1], false);
        assert!(m[1] > 0.5 && m[3] > 0.5 && m[0] < 0.5 && m[2] < 0.5, "{m:?}");
        // No correction: every motor at the throttle, and the average stays the throttle.
        assert_eq!(mix(0.3, [0.0; 3], false), [0.3; 4]);
        let m = mix(0.5, [0.1, -0.05, 0.02], false);
        assert!((m.iter().sum::<f32>() / 4.0 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn airmode_keeps_authority_at_both_ends_of_the_throttle() {
        let at_zero = mix(0.0, [0.2, 0.0, 0.0], false);
        assert!(at_zero.iter().all(|v| (0.0..=1.0).contains(v)));
        assert!((at_zero[3] - at_zero[0] - 0.4).abs() < 1e-6, "full correction at zero throttle: {at_zero:?}");
        let at_full = mix(1.0, [0.2, 0.0, 0.0], false);
        assert!((at_full[3] - at_full[0] - 0.4).abs() < 1e-6, "full correction at full throttle: {at_full:?}");
        // Corrections wider than the motors' range are scaled to fit.
        let huge = mix(0.5, [0.8, 0.0, 0.0], false);
        assert!((huge[3] - 1.0).abs() < 1e-6 && huge[0].abs() < 1e-6, "{huge:?}");
    }

    #[test]
    fn three_d_mixes_in_the_reversed_half() {
        let m = mix(-0.5, [0.1, 0.0, 0.0], true);
        assert!(m.iter().all(|v| (-1.0..=0.0).contains(v)), "{m:?}");
        // Higher (less negative) still means more upward thrust, so the same signs turn it.
        assert!(m[3] > m[0]);
    }

    #[test]
    fn rate_loop_pushes_toward_the_setpoint_and_relaxes_i_in_flips() {
        let cfg = PidConfig::default();
        let mut pid = RateController::default();
        let out = pid.update(&cfg, [200.0, -100.0, 50.0], [0.0; 3], 0.5, 0.002);
        assert!(out[0] > 0.0 && out[1] < 0.0 && out[2] > 0.0, "{out:?}");
        // A steady error builds I.
        let mut steady = RateController::default();
        for _ in 0..200 {
            steady.update(&cfg, [30.0, 0.0, 0.0], [0.0; 3], 0.5, 0.002);
        }
        assert!(steady.iterm()[0] > 0.0);
        // A snapped stick (a flip) barely builds any.
        let mut flip = RateController::default();
        flip.update(&cfg, [0.0; 3], [0.0; 3], 0.5, 0.002);
        for _ in 0..20 {
            flip.update(&cfg, [800.0, 0.0, 0.0], [0.0; 3], 0.5, 0.002);
        }
        let mut plain = RateController::default();
        plain.update(&cfg, [800.0, 0.0, 0.0], [0.0; 3], 0.5, 0.002);
        for _ in 0..20 {
            plain.update(&cfg, [800.0, 0.0, 0.0], [0.0; 3], 0.5, 0.002);
        }
        assert!(flip.iterm()[0] < plain.iterm()[0] * 0.5, "relax {} vs {}", flip.iterm()[0], plain.iterm()[0]);
    }

    #[test]
    fn no_i_term_before_the_throttle_comes_up() {
        let cfg = PidConfig::default();
        let mut pid = RateController::default();
        for _ in 0..100 {
            pid.update(&cfg, [30.0, 0.0, 0.0], [0.0; 3], 0.05, 0.002);
        }
        assert_eq!(pid.iterm(), [0.0; 3]);
        assert!(!pid.airmode_active);
    }
}
