// 3D vector and Quaternion math for FPV Horizon simulation.
// Implements vector arithmetic, projections, and quaternion rotations matching JOML / Minecraft conventions.

use std::f32::consts::PI;

/// A 3D vector of single-precision floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN: Self = Self { x: 0.0, y: -1.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: -1.0 };
    pub const BACK: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const LEFT: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const RIGHT: Self = Self { x: -1.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn add(self, o: Self) -> Self {
        Self { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z }
    }

    pub fn sub(self, o: Self) -> Self {
        Self { x: self.x - o.x, y: self.y - o.y, z: self.z - o.z }
    }

    pub fn mul(self, scalar: f32) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar, z: self.z * scalar }
    }

    pub fn dot(self, o: Self) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Self) -> Self {
        Self {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized_or_zero(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq > 1e-12 {
            let inv = 1.0 / len_sq.sqrt();
            self.mul(inv)
        } else {
            Self::ZERO
        }
    }

    pub fn angle_between(self, o: Self) -> f32 {
        let denom = (self.length_squared() * o.length_squared()).sqrt();
        if denom < 1e-12 {
            return 0.0;
        }
        let dot = (self.dot(o) / denom).clamp(-1.0, 1.0);
        dot.acos()
    }

    pub fn project(self, on: Self) -> Self {
        let on_len_sq = on.length_squared();
        if on_len_sq < 1e-12 {
            Self::ZERO
        } else {
            on.mul(self.dot(on) / on_len_sq)
        }
    }

    pub fn fma(self, scalar: f32, addend: Self) -> Self {
        Self {
            x: self.x + scalar * addend.x,
            y: self.y + scalar * addend.y,
            z: self.z + scalar * addend.z,
        }
    }
}

impl std::ops::Add for Vec3f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { self.add(rhs) }
}

impl std::ops::Sub for Vec3f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { self.sub(rhs) }
}

impl std::ops::Mul<f32> for Vec3f {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self { self.mul(rhs) }
}

/// A quaternion (x, y, z, w) representing 3D rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: Vec3f, angle_rad: f32) -> Self {
        let half = angle_rad * 0.5;
        let s = half.sin();
        let norm_axis = axis.normalized_or_zero();
        Self {
            x: norm_axis.x * s,
            y: norm_axis.y * s,
            z: norm_axis.z * s,
            w: half.cos(),
        }
    }

    /// Rotation with Euler order YXZ (Yaw around Y, Pitch around X, Roll around Z).
    /// Matches JOML's `rotationYXZ(angleY, angleX, angleZ)`.
    pub fn rotation_y_x_z(yaw: f32, pitch: f32, roll: f32) -> Self {
        let qy = Self::from_axis_angle(Vec3f::new(0.0, 1.0, 0.0), yaw);
        let qx = Self::from_axis_angle(Vec3f::new(1.0, 0.0, 0.0), pitch);
        let qz = Self::from_axis_angle(Vec3f::new(0.0, 0.0, 1.0), roll);
        qy.mul(qx).mul(qz)
    }

    pub fn mul(self, o: Self) -> Self {
        Self {
            w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
            x: self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            y: self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            z: self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
        }
    }

    pub fn rotate_x(self, angle_rad: f32) -> Self {
        let q = Self::from_axis_angle(Vec3f::new(1.0, 0.0, 0.0), angle_rad);
        self.mul(q)
    }

    pub fn rotate_y(self, angle_rad: f32) -> Self {
        let q = Self::from_axis_angle(Vec3f::new(0.0, 1.0, 0.0), angle_rad);
        self.mul(q)
    }

    pub fn rotate_z(self, angle_rad: f32) -> Self {
        let q = Self::from_axis_angle(Vec3f::new(0.0, 0.0, 1.0), angle_rad);
        self.mul(q)
    }

    pub fn rotate_axis(self, angle_rad: f32, axis: Vec3f) -> Self {
        let q = Self::from_axis_angle(axis, angle_rad);
        self.mul(q)
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn normalize(self) -> Self {
        let len_sq = self.length_squared();
        if len_sq > 1e-12 {
            let inv = 1.0 / len_sq.sqrt();
            Self {
                x: self.x * inv,
                y: self.y * inv,
                z: self.z * inv,
                w: self.w * inv,
            }
        } else {
            Self::IDENTITY
        }
    }

    /// Transforms a 3D vector by this quaternion rotation.
    pub fn transform_vec3(self, v: Vec3f) -> Vec3f {
        let qv = Vec3f::new(self.x, self.y, self.z);
        let uv = qv.cross(v);
        let uuv = qv.cross(uv);
        v.add((uv.mul(self.w).add(uuv)).mul(2.0))
    }

    /// Spherical linear interpolation between two quaternions.
    pub fn slerp(self, mut target: Self, alpha: f32) -> Self {
        let mut dot = self.x * target.x + self.y * target.y + self.z * target.z + self.w * target.w;
        if dot < 0.0 {
            target.x = -target.x;
            target.y = -target.y;
            target.z = -target.z;
            target.w = -target.w;
            dot = -dot;
        }

        if dot > 0.9995 {
            // Linear interpolation if too close to avoid division by zero
            let x = self.x + alpha * (target.x - self.x);
            let y = self.y + alpha * (target.y - self.y);
            let z = self.z + alpha * (target.z - self.z);
            let w = self.w + alpha * (target.w - self.w);
            return Self::new(x, y, z, w).normalize();
        }

        let theta_0 = dot.clamp(-1.0, 1.0).acos();
        let theta = theta_0 * alpha;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Self {
            x: s0 * self.x + s1 * target.x,
            y: s0 * self.y + s1 * target.y,
            z: s0 * self.z + s1 * target.z,
            w: s0 * self.w + s1 * target.w,
        }.normalize()
    }

    /// Decomposes quaternion into Tait-Bryan angles with rotation order YXZ.
    /// Returns [yaw, pitch, roll] in radians.
    pub fn to_euler_angles_yxz(self) -> [f32; 3] {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        let sin_p = 2.0 * (w * x - y * z);
        let pitch = if sin_p.abs() >= 1.0 {
            (PI / 2.0).copysign(sin_p)
        } else {
            sin_p.asin()
        };

        let yaw = (2.0 * (w * y + x * z)).atan2(1.0 - 2.0 * (x * x + y * y));
        let roll = (2.0 * (w * z + x * y)).atan2(1.0 - 2.0 * (x * x + z * z));

        [yaw, pitch, roll]
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3f::new(1.0, 2.0, 3.0);
        let v2 = Vec3f::new(4.0, 5.0, 6.0);
        assert_eq!(v1.add(v2), Vec3f::new(5.0, 7.0, 9.0));
        assert_eq!(v1.dot(v2), 32.0);
        assert_eq!(v1.mul(2.0), Vec3f::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_quat_rotation_identity() {
        let q = Quat::IDENTITY;
        let v = Vec3f::new(1.0, 2.0, 3.0);
        let rotated = q.transform_vec3(v);
        assert!((rotated.x - v.x).abs() < 1e-5);
        assert!((rotated.y - v.y).abs() < 1e-5);
        assert!((rotated.z - v.z).abs() < 1e-5);
    }

    #[test]
    fn test_quat_rotation_90_deg_y() {
        let q = Quat::from_axis_angle(Vec3f::UP, PI / 2.0);
        let v = Vec3f::new(0.0, 0.0, -1.0); // Facing forward
        let rotated = q.transform_vec3(v);
        // Rotated 90 degrees around Y: forward (-Z) should become left (-X)
        assert!((rotated.x - (-1.0)).abs() < 1e-5);
        assert!((rotated.z - 0.0).abs() < 1e-5);
    }

    #[test]
    fn test_quat_euler_decomposition() {
        let q = Quat::rotation_y_x_z(0.3, 0.4, 0.5);
        let [yaw, pitch, roll] = q.to_euler_angles_yxz();
        assert!((yaw - 0.3).abs() < 1e-4);
        assert!((pitch - 0.4).abs() < 1e-4);
        assert!((roll - 0.5).abs() < 1e-4);
    }
}
