use core::ops::{Add, Sub, Mul, Div, Neg};

// Float is deprecated, the math objects will only make use of f32 (if precision is needed) and integer variables only. 
// Only two types of floats are going to be used accross many of the math objects such as vectors and quaternion.
// `From<i16>` is the widest integer conversion both f32 and f64 implement, so `T::from(2)` is exact for either.
// Non-integer constants go through `from_f64`, because `f32: From<f64>` does not exist (it would be lossy).
pub trait Float: Copy + PartialOrd + From<i16> + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self>  + Neg<Output = Self>{
    const PI:                       Self;
    const INFINITY:                 Self;
    const NEG_INFINITY:             Self;
    fn from_f64(value: f64)     ->  Self;
    fn sin(self)                ->  Self;
    fn cos(self)                ->  Self;
    fn asin(self)               ->  Self;
    fn atan2(self, x: Self)     ->  Self;
    fn sqrt(self)               ->  Self;
    fn abs(self)                ->  Self;
    fn copysign(self, y: Self)  ->  Self;
    fn to_degrees(self)         ->  Self;
}

impl Float for f32 {
    const PI:                       Self = core::f32::consts::PI;
    const INFINITY:                 Self = f32::INFINITY;
    const NEG_INFINITY:             Self = f32::NEG_INFINITY;
    fn from_f64(value: f64)     ->  Self { value as Self }
    fn sin(self)                ->  Self { libm::sinf(self) }
    fn cos(self)                ->  Self { libm::cosf(self) }
    fn asin(self)               ->  Self { libm::asinf(self) }
    fn atan2(self, x: Self)     ->  Self { libm::atan2f(self, x) }
    fn sqrt(self)               ->  Self { libm::sqrtf(self) }
    fn abs(self)                ->  Self { libm::fabsf(self) }
    fn copysign(self, y: Self)  ->  Self { libm::copysignf(self, y) }
    fn to_degrees(self)         ->  Self { self * (180.0 / core::f32::consts::PI) }
}

impl Float for f64 {
    const PI:                       Self = core::f64::consts::PI;
    const INFINITY:                 Self = f64::INFINITY;
    const NEG_INFINITY:             Self = f64::NEG_INFINITY;
    fn from_f64(value: f64)     ->  Self { value }
    fn sin(self)                ->  Self { libm::sin(self) }
    fn cos(self)                ->  Self { libm::cos(self) }
    fn asin(self)               ->  Self { libm::asin(self) }
    fn atan2(self, x: Self)     ->  Self { libm::atan2(self, x) }
    fn sqrt(self)               ->  Self { libm::sqrt(self) }
    fn abs(self)                ->  Self { libm::fabs(self) }
    fn copysign(self, y: Self)  ->  Self { libm::copysign(self, y) }
    fn to_degrees(self)         ->  Self { self * (180.0 / core::f64::consts::PI) }
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

// A custom 2D Vector.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2<f32> {
    pub x: f32,
    pub y: f32,
}

impl Vector2<f32> {
    pub fn new(x: f32, y: f32) -> Self {
        Self {x, y}
    }

    pub fn mag(self) -> f32 {
        let x: f32 = self.x;
        let y: f32 = self.y;
        (x * x + y * y).sqrt()
    }

    pub fn norm(self) -> Vector2<f32> {
        let mag = self.mag();

        if mag == 0.0 {
            Vector2 {x: 0.0, y: 0.0}
        } else {
            Vector2 {x: self.x / mag, y: self.y / mag}
        }
    }

    pub fn unit(self) -> Vector2<f32> {
        let zero = 0.0;
        let norm_vector = self.norm();
        let mut unit_vector = Vector2{x: zero, y: zero};

        if norm_vector.x > zero {
            unit_vector.x = 1.0;
        } else if norm_vector.x < zero {
            unit_vector.x = -1.0;
        }

        if norm_vector.y > zero {
            unit_vector.y = 1.0;
        } else if norm_vector.y < zero {
            unit_vector.y = -1.0;
        }

        unit_vector
    }

    pub fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    pub fn cross(self, rhs: Self) -> f32 {
        self.x*rhs.y - self.y*rhs.x
    }
}

impl Add for Vector2<f32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2<f32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Vector2<f32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Div for Vector2<f32> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

// Pre-defined types.
//pub type F32Vector2 = Vector2<f32>;
//pub type F64Vector2 = Vector2<f64>;


// A custom 3D Vector.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector3<f32> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3<f32> {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {x, y, z}
    }

    pub fn mag(self) -> f32 {
        let x: f32 = self.x;
        let y: f32 = self.y;
        let z: f32 = self.z;
        (x * x + y * y + z * z).sqrt()
    }

    pub fn norm(self) -> Vector3<f32> {
        let mag = self.mag();

        if mag == 0.0 {
            Vector3 {x: 0.0, y: 0.0, z: 0.0}
        } else {
            Vector3 {x: self.x / mag, y: self.y / mag, z: self.z / mag}
        }
    }

    pub fn unit(self) -> Vector3<f32> {
        let zero = 0.0;
        let norm_vector = self.norm();
        let mut unit_vector = Vector3{x: zero, y: zero, z: zero};

        if norm_vector.x > zero {
            unit_vector.x = 1.0;
        } else if norm_vector.x < zero {
            unit_vector.x = -1.0;
        }

        if norm_vector.y > zero {
            unit_vector.y = 1.0;
        } else if norm_vector.y < zero {
            unit_vector.y = -1.0;
        }

        if norm_vector.z > zero {
            unit_vector.z = 1.0;
        } else if norm_vector.z < zero {
            unit_vector.z = -1.0;
        }

        unit_vector
    }

    pub fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn cross(self, rhs: Self) -> Vector3<f32> {
        Vector3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl Add for Vector3<f32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3<f32> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vector3<f32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Div for Vector3<f32> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

// Pre-defined types.
//pub type F32Vector3 = Vector3<f32>;
//pub type F64Vector3 = Vector3<f64>;



#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Basis<Vector3> {
    pub x: Vector3,
    pub y: Vector3,
    pub z: Vector3,
}




#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Quaternion<f32> {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion<f32> {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {x, y, z, w}
    }

    pub fn new_from_axis_angle(angle: f32, axis: Vector3<f32>) -> Self {
        let half = angle / 2.0;
        let half_sin = half.sin();

        Self {
            x: axis.x * half_sin,
            y: axis.y * half_sin,
            z: axis.z * half_sin,
            w: half.cos(),
        }
    }

    pub fn mag(self) -> f32 {
        let x: f32 = self.x;
        let y: f32 = self.y;
        let z: f32 = self.z;
        let w: f32 = self.w;
        (x * x + y * y + z * z + w * w).sqrt()
    }

    pub fn norm(self) -> Quaternion<f32> {
        let mag = self.mag();

        if mag == 0.0 {
            Quaternion {x: 0.0, y: 0.0, z: 0.0, w: 1.0}
        } else {
            Quaternion {x: self.x / mag, y: self.y / mag, z: self.z / mag, w: self.w / mag}
        }
    }

    pub fn conj(self) -> Quaternion<f32> {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn mult(self, rhs: Self) -> Quaternion<f32> {
        Quaternion {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }

    pub fn rotate(self, angle: f32, axis: Vector3<f32>) -> Quaternion<f32> {
        Quaternion { x: self.x, y: self.y, z: self.z, w: self.w }.mult(Quaternion::new_from_axis_angle(angle, axis))
    }

    pub fn get_euler_angles(self) -> Vector3<f32> {
        let one = 1.0;
        let two = 2.0;
        let pi = core::f32::consts::PI;

        let sinr_cosp = two * (self.w * self.x + self.y * self.z);
        let cosr_cosp = one - two * (self.x * self.x + self.y * self.y);
        let roll_x = sinr_cosp.atan2(cosr_cosp);

        // Straight up at the poles asin() is undefined, so clamp to +/- 90 degrees instead.
        let sinp = two * (self.w * self.y - self.z * self.x);
        let pitch_y = if sinp.abs() >= one {
            (pi / two).copysign(sinp)
        } else {
            clamp(sinp, -one, one).asin()
        };

        let siny_cosp = two * (self.w * self.z + self.x * self.y);
        let cosy_cosp = one - two * (self.y * self.y + self.z * self.z);
        let yaw_z = siny_cosp.atan2(cosy_cosp);

        Vector3::new(roll_x.to_degrees(), pitch_y.to_degrees(), yaw_z.to_degrees())
    }

    pub fn get_basis(self) -> Basis<Vector3<f32>> {
        let one = 1.0;
        let two = 2.0;

        Basis {
            x: Vector3::new(
                one - two * (self.y * self.y + self.z * self.z),
                two * (self.x * self.y - self.w * self.z),
                two * (self.x * self.z + self.w * self.y),
            ),
            y: Vector3::new(
                two * (self.x * self.y + self.w * self.z),
                one - two * (self.x * self.x + self.z * self.z),
                two * (self.y * self.z - self.w * self.x),
            ),
            z: Vector3::new(
                two * (self.x * self.z - self.w * self.y),
                two * (self.y * self.z + self.w * self.x),
                one - two * (self.x * self.x + self.y * self.y),
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

impl Add for Quaternion<f32> {
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

impl Sub for Quaternion<f32> {
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

impl Mul for Quaternion<f32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl Div for Quaternion<f32> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }
}



// A PID controller. Unlike the vectors and the quaternion this one carries state between calls,
// so a SIL keeps one instance alive and steps it once per loop with the measurement and the
// elapsed time.
#[derive(Debug, Clone, PartialEq)]
pub struct PID<f32> {
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

impl PID<f32> {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        let zero: f32 = 0.0;

        Self {
            kp,
            ki,
            kd,
            g: 1.0,

            proportional: zero,
            integral: zero,
            differential: zero,

            integral_min: f32::NEG_INFINITY,
            integral_max: f32::INFINITY,

            set_point: zero,
            output: zero,
            error: zero,
            previous_error: zero,

            min_dt: 0.001,
        }
    }

    pub fn with_gain(mut self, g: f32) -> Self {
        self.g = g;
        self
    }

    pub fn with_set_point(mut self, set_point: f32) -> Self {
        self.set_point = set_point;
        self
    }

    pub fn with_integral_clamp(mut self, min: f32, max: f32) -> Self {
        self.set_integral_clamp(min, max);
        self
    }

    pub fn with_min_dt(mut self, min_dt: f32) -> Self {
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
        let zero = 0.0;

        self.proportional = zero;
        self.integral = zero;
        self.differential = zero;
        self.error = zero;
        self.previous_error = zero;
        self.output = zero;
    }
}

// Pre-defined types.
//pub type F32PID = PID<f32>;
//pub type F64PID = PID<f64>;