use core::ops::{Add, Sub, Mul, Div};

pub mod scalar {
    pub const PI: f32 = core::f32::consts::PI;

    pub fn sin(x: f32)              -> f32 { libm::sinf(x) }
    pub fn cos(x: f32)              -> f32 { libm::cosf(x) }
    pub fn asin(x: f32)             -> f32 { libm::asinf(x) }
    pub fn acos(x: f32)             -> f32 { libm::acosf(x) }
    pub fn atan2(y: f32, x: f32)    -> f32 { libm::atan2f(y, x) }
    pub fn sqrt(x: f32)             -> f32 { libm::sqrtf(x) }
    pub fn abs(x: f32)              -> f32 { libm::fabsf(x) }
    pub fn copysign(x: f32, y: f32) -> f32 { libm::copysignf(x, y) }

    pub fn to_degrees(x: f32) -> f32 { x * (180.0 / PI) }
    pub fn to_radians(x: f32) -> f32 { x * (PI / 180.0) }
}

pub fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

// The per-component sign used by `unit`, returning zero for zero rather than the +1 that
// `copysign` would give.
fn axis_sign(value: f32) -> f32 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    }
}

// A custom 2D Vector.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn mag(self) -> f32 {
        scalar::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn norm(self) -> Self {
        let mag = self.mag();

        if mag == 0.0 {
            Self::ZERO
        } else {
            Self { x: self.x / mag, y: self.y / mag }
        }
    }

    pub fn unit(self) -> Self {
        Self { x: axis_sign(self.x), y: axis_sign(self.y) }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Vector2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div for Vector2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}


// A custom 3D Vector.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn mag(self) -> f32 {
        scalar::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn norm(self) -> Self {
        let mag = self.mag();

        if mag == 0.0 {
            Self::ZERO
        } else {
            Self { x: self.x / mag, y: self.y / mag, z: self.z / mag }
        }
    }

    pub fn unit(self) -> Self {
        Self {
            x: axis_sign(self.x),
            y: axis_sign(self.y),
            z: axis_sign(self.z),
        }
    }

    pub fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Div for Vector3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}


// Three vectors defining a coordinate frame. Currently produced by `Quaternion::get_basis`; the
// operations on it (identity, transpose, vector product) are still to come.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Basis {
    pub x: Vector3,
    pub y: Vector3,
    pub z: Vector3,
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    // The axis is normalized first, so a caller passing an unnormalized direction still gets a
    // unit quaternion back.
    pub fn new_from_axis_angle(angle: f32, axis: Vector3) -> Self {
        let axis = axis.norm();
        let half = angle / 2.0;
        let half_sin = scalar::sin(half);

        Self {
            x: axis.x * half_sin,
            y: axis.y * half_sin,
            z: axis.z * half_sin,
            w: scalar::cos(half),
        }
    }

    pub fn mag(self) -> f32 {
        scalar::sqrt(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w)
    }

    pub fn norm(self) -> Self {
        let mag = self.mag();

        if mag == 0.0 {
            Self::IDENTITY
        } else {
            Self { x: self.x / mag, y: self.y / mag, z: self.z / mag, w: self.w / mag }
        }
    }

    pub fn conj(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    // Hamilton product: the composition of two rotations. Not component-wise, which is why it is a
    // named method and `Mul` is left unimplemented for two quaternions.
    pub fn mult(self, rhs: Self) -> Self {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }

    pub fn rotate(self, angle: f32, axis: Vector3) -> Self {
        self.mult(Self::new_from_axis_angle(angle, axis))
    }

    pub fn get_euler_angles(self) -> Vector3 {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll_x = scalar::atan2(sinr_cosp, cosr_cosp);

        // Straight up at the poles asin() is undefined, so clamp to +/- 90 degrees instead.
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch_y = if scalar::abs(sinp) >= 1.0 {
            scalar::copysign(scalar::PI / 2.0, sinp)
        } else {
            scalar::asin(clamp(sinp, -1.0, 1.0))
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw_z = scalar::atan2(siny_cosp, cosy_cosp);

        Vector3::new(
            scalar::to_degrees(roll_x),
            scalar::to_degrees(pitch_y),
            scalar::to_degrees(yaw_z),
        )
    }

    pub fn get_basis(self) -> Basis {
        Basis {
            x: Vector3::new(
                1.0 - 2.0 * (self.y * self.y + self.z * self.z),
                2.0 * (self.x * self.y - self.w * self.z),
                2.0 * (self.x * self.z + self.w * self.y),
            ),
            y: Vector3::new(
                2.0 * (self.x * self.y + self.w * self.z),
                1.0 - 2.0 * (self.x * self.x + self.z * self.z),
                2.0 * (self.y * self.z - self.w * self.x),
            ),
            z: Vector3::new(
                2.0 * (self.x * self.z - self.w * self.y),
                2.0 * (self.y * self.z + self.w * self.x),
                1.0 - 2.0 * (self.x * self.x + self.y * self.y),
            ),
        }
    }

    // Wolfram, R = [X, Y, Z], X = (1, 0, 0), Y = (0, 1, 0), Z = (0, 0, 1)
    pub fn get_rotation_matrix(self) -> [[f32; 3]; 3] {
        let basis = self.get_basis();

        [
            [basis.x.x, basis.x.y, basis.x.z],  // Right, X
            [basis.y.x, basis.y.y, basis.y.z],  // Up, Y
            [basis.z.x, basis.z.y, basis.z.z],  // Front, Z
        ]
    }
}

impl Add for Quaternion {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Sub for Quaternion {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}


// A PID controller. Unlike the vectors and the quaternion this one carries state between calls,
// so a SIL keeps one instance alive and steps it once per loop with the measurement and the
// elapsed time.
#[derive(Debug, Clone, PartialEq)]
pub struct PID {
    // Gains.
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub g: f32,

    // Terms of the last update, kept public so a SIL can log or plot them.
    pub proportional: f32,
    pub integral: f32,
    pub differential: f32,

    // Anti-windup limits on the integral term.
    pub integral_min: f32,
    pub integral_max: f32,

    pub set_point: f32,
    pub output: f32,
    pub error: f32,
    pub previous_error: f32,

    // Lower bound on dt, so a stalled or repeated frame cannot divide by zero.
    pub min_dt: f32,
}

impl PID {
    pub const fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            g: 1.0,

            proportional: 0.0,
            integral: 0.0,
            differential: 0.0,

            integral_min: f32::NEG_INFINITY,
            integral_max: f32::INFINITY,

            set_point: 0.0,
            output: 0.0,
            error: 0.0,
            previous_error: 0.0,

            min_dt: 0.001,
        }
    }

    pub const fn with_gain(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    pub const fn with_set_point(mut self, set_point: f32) -> Self {
        self.set_point = set_point;
        self
    }

    pub const fn with_integral_clamp(mut self, min: f32, max: f32) -> Self {
        self.integral_min = min;
        self.integral_max = max;
        self
    }

    pub const fn with_min_dt(mut self, min_dt: f32) -> Self {
        self.min_dt = min_dt;
        self
    }

    pub fn update(&mut self, measured_value: f32, dt: f32) -> f32 {
        let dt = if dt < self.min_dt { self.min_dt } else { dt };

        self.error = self.set_point - measured_value;
        self.proportional = self.error;
        self.integral = clamp(
            self.integral + self.error * dt, self.integral_min, self.integral_max
        );
        self.differential = (self.error - self.previous_error) / dt;

        self.output = self.g * (
            self.proportional * self.kp
            + self.integral * self.ki
            + self.differential * self.kd
        );

        self.previous_error = self.error;

        self.output
    }

    pub fn set_integral_clamp(&mut self, min: f32, max: f32) {
        self.integral_min = min;
        self.integral_max = max;
    }

    pub fn reset(&mut self) {
        self.proportional = 0.0;
        self.integral = 0.0;
        self.differential = 0.0;
        self.error = 0.0;
        self.previous_error = 0.0;
        self.output = 0.0;
    }
}
