import math


def clamp(n, min, max):
    if n < min:
        return min
    elif n > max:
        return max
    else:
        return n


def map(n, from_min, from_max, to_min, to_max):
    return (n - from_min) * (to_max - to_min) / (from_max - from_min) + to_min


def sin(n):
    return math.sin(n)


def cos(n):
    return math.cos(n)


def acos(n):
    return math.acos(n)


def deadzone(n, deadzone):
    if abs(n) < deadzone:
        return 0
    else:
        return n


def low_pass(a, value, new):
    return new * a + value * (1 - a)


def amplitude_impedance(f, value, new):
    if new > value * f:
        return value
    else:
        return new


# Vector with two components.
class Vector2:
    def __init__(self, x, y):
        self.x = x
        self.y = y

        self.n = 2

    def _get_val(self, vector):
        return vector.magnitude() if isinstance(vector, Vector2) else vector

    def __add__(self, vector: Vector2):
        return Vector2(self.x + vector.x, self.y + vector.y)

    def __sub__(self, vector: Vector2):
        return Vector2(self.x - vector.x, self.y - vector.y)

    def __mul__(self, vector: Vector2):
        return Vector2(self.x * vector.x, self.y * vector.y)

    def __div__(self, vector: Vector2):
        return Vector2(self.x / vector.x, self.y / vector.y)

    def __truediv__(self, vector: Vector2):
        return Vector2(self.x / vector.x, self.y / vector.y)

    def __str__(self):
        return f"({self.x:.3f},{self.y:.3f})"

    def __eq__(self, vector):
        return self.magnitude() == self._get_val(vector)

    def __lt__(self, vector: Vector2):
        return self.magnitude() < self._get_val(vector)

    def magnitude(self):
        return (self.x**2 + self.y**2) ** (1 / 2)

    def median(self):
        return (self.x + self.y) / self.n

    def absmedian(self):
        return abs(self.median())

    def normal(self):
        magnitude = self.magnitude()

        if magnitude == 0:
            return Vector2(0, 0)

        return Vector2(self.x / magnitude, self.y / magnitude)

    def unit(self):
        unit_vector = self.normal()

        if unit_vector.x > 0:
            unit_vector.x = 1
        elif unit_vector.x < 0:
            unit_vector.x = -1
        else:
            unit_vector.x = 0

        if unit_vector.y > 0:
            unit_vector.y = 1
        elif unit_vector.y < 0:
            unit_vector.y = -1
        else:
            unit_vector.y = 0

        return unit_vector

    def dot(self, vector: Vector2):
        return self.x * vector.x + self.y * vector.y

    def cross(self, vector: Vector2):
        return self.x * vector.y - self.y * vector.x

    def clone(self):
        return Vector2(self.x, self.y)


# Vector with 3 components.
class Vector3:
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

        self.n = 3

    def _get_val(self, vector):
        return vector.magnitude() if isinstance(vector, Vector3) else vector

    def __add__(self, vector: Vector3):
        return Vector3(self.x + vector.x, self.y + vector.y, self.z + vector.z)

    def __sub__(self, vector: Vector3):
        return Vector3(self.x - vector.x, self.y - vector.y, self.z - vector.z)

    def __mult__(self, vector: Vector3):
        return Vector3(self.x * vector.x, self.y * vector.y, self.z * vector.z)

    def __div__(self, vector: Vector3):
        return Vector3(self.x / vector.x, self.y / vector.y, self.z / vector.z)

    def __truediv__(self, vector: Vector3):
        return Vector3(self.x / vector.x, self.y / vector.y, self.z / vector.z)

    def __str__(self):
        return f"({self.x:.3f},{self.y:.3f},{self.z:.3f})"

    def __eq__(self, vector):
        return self.magnitude() == self._get_val(vector)

    def __lt__(self, vector: Vector3):
        return self.magnitude() < self._get_val(vector)

    def magnitude(self):
        return (self.x**2 + self.y**2 + self.z**2) ** (1 / 2)

    def median(self):
        return (self.x + self.y + self.z) / self.n

    def absmedian(self):
        return abs(self.median())

    def normal(self):
        magnitude = self.magnitude()
        if magnitude == 0:
            return Vector3(0, 0, 0)

        return Vector3(self.x / magnitude, self.y / magnitude, self.z / magnitude)

    def unit(self):
        unit_vector = self.normal()

        if unit_vector.x > 0:
            unit_vector.x = 1
        elif unit_vector.x < 0:
            unit_vector.x = -1
        else:
            unit_vector.x = 0

        if unit_vector.y > 0:
            unit_vector.y = 1
        elif unit_vector.y < 0:
            unit_vector.y = -1
        else:
            unit_vector.y = 0

        if unit_vector.z > 0:
            unit_vector.z = 1
        elif unit_vector.z < 0:
            unit_vector.z = -1
        else:
            unit_vector.z = 0

        return unit_vector

    def dot(self, vector: Vector3):
        return self.x * vector.x + self.y * vector.y + self.z * vector.z

    def cross(self, vector: Vector3):
        return (
            self.y * vector.z
            - self.z * vector.y * self.z * vector.x
            - self.x * vector.z * self.x * vector.y
            - self.y * vector.x
        )

    def clone(self):
        return Vector3(self.x, self.y, self.z)


# A vector matrix composed of three 3D Vectors that is used to define a coordinate system. Generally a local coordinate system of a component.
class Basis:
    def __init__(self, x: Vector3, y: Vector3, z: Vector3):
        self.x: Vector3 = x
        self.y: Vector3 = y
        self.z: Vector3 = z

        self.n = 3 * 3


# Quaternion is a quaternion.
class Quaternion:
    def __init__(self, x, y, z, w):
        self.x = x
        self.y = y
        self.z = z
        self.w = w

        self.n = 4

    @staticmethod
    def from_axis_angle(angle, axis: Vector3):
        axis = axis.normal()
        return Quaternion(
            axis.x * math.sin(angle / 2),
            axis.y * math.sin(angle / 2),
            axis.z * math.sin(angle / 2),
            math.cos(angle / 2),
        )

    def _get_val(self, quaternion):
        return (
            quaternion.magnitude() if isinstance(quaternion, Quaternion) else quaternion
        )

    def __add__(self, other):
        return Quaternion(
            self.x + other.x, self.y + other.y, self.z + other.z, self.w + other.w
        )

    def __sub__(self, other):
        return Quaternion(
            self.x - other.x, self.y - other.y, self.z - other.z, self.w - other.w
        )

    def __mul__(self, other):
        return Quaternion(
            self.x * other, self.y * other, self.z * other, self.w * other
        )

    def __div__(self, other):
        return Quaternion(
            self.x / other, self.y / other, self.z / other, self.w / other
        )

    def __truediv__(self, other):
        return Quaternion(
            self.x / other, self.y / other, self.z / other, self.w / other
        )

    def __str__(self):
        return f"({self.x:.3f},{self.y:.3f},{self.z:.3f},{self.w:.3f})"

    def __eq__(self, quaternion):
        return self.magnitude() == self._get_val(quaternion)

    def __lt__(self, quaternion: Quaternion):
        return self.magnitude() < self._get_val(quaternion)

    def magnitude(self):
        return (self.x**2 + self.y**2 + self.z**2 + self.w**2) ** (1 / 2)

    def median(self):
        return (self.x + self.y + self.z + self.w) / self.n

    def absmedian(self):
        return abs(self.median())

    def normal(self):
        magnitude = self.magnitude()
        if magnitude == 0:
            return Quaternion(0, 0, 0, 1)

        return Quaternion(
            self.x / magnitude,
            self.y / magnitude,
            self.z / magnitude,
            self.w / magnitude,
        )

    def conjugate(self):
        return Quaternion(-self.x, -self.y, -self.z, self.w)

    def mult(self, quaternion: Quaternion):
        w = (
            self.w * quaternion.w
            - self.x * quaternion.x
            - self.y * quaternion.y
            - self.z * quaternion.z
        )
        x = (
            self.w * quaternion.x
            + self.x * quaternion.w
            + self.y * quaternion.z
            - self.z * quaternion.y
        )
        y = (
            self.w * quaternion.y
            - self.x * quaternion.z
            + self.y * quaternion.w
            + self.z * quaternion.x
        )
        z = (
            self.w * quaternion.z
            + self.x * quaternion.y
            - self.y * quaternion.x
            + self.z * quaternion.w
        )
        return Quaternion(x, y, z, w)

    def rotate(self, angle, axis: Vector3):
        return Quaternion(self.x, self.y, self.z, self.w).mult(
            Quaternion.from_axis_angle(angle, axis)
        )

    def get_euler_angles(self):
        sinr_cosp = 2 * (self.w * self.x + self.y * self.z)
        cosr_cosp = 1 - 2 * (self.x**2 + self.y**2)
        roll_x = math.atan2(sinr_cosp, cosr_cosp)

        sinp = 2 * (self.w * self.y - self.z * self.x)
        if abs(sinp) >= 1:
            pitch_y = math.copysign(math.pi / 2, sinp)
        else:
            pitch_y = math.asin(clamp(sinp, -1.0, 1.0))

        siny_cosp = 2 * (self.w * self.z + self.x * self.y)
        cosy_cosp = 1 - 2 * (self.y**2 + self.z**2)
        yaw_z = math.atan2(siny_cosp, cosy_cosp)

        return Vector3(math.degrees(roll_x), math.degrees(pitch_y), math.degrees(yaw_z))

    def GetBasis(self):
        return Basis(
            Vector3(
                1 - 2 * (self.y**2 + self.z**2),
                2 * (self.x * self.y - self.w * self.z),
                2 * (self.x * self.z + self.w * self.y),
            ),
            Vector3(
                2 * (self.x * self.y + self.w * self.z),
                1 - 2 * (self.x**2 + self.z**2),
                2 * (self.y * self.z - self.w * self.x),
            ),
            Vector3(
                2 * (self.x * self.z - self.w * self.y),
                2 * (self.y * self.z + self.w * self.x),
                1 - 2 * (self.x**2 + self.y**2),
            ),
        )

    def GetRotationMatrix(self):
        # Wolfram, R = [X, Y, Z], X = (1, 0, 0), Y = (0, 1, 0), Z = (0, 0, 1)
        return [
            [
                1 - 2 * (self.y**2 + self.z**2),
                2 * (self.x * self.y - self.w * self.z),
                2 * (self.x * self.z + self.w * self.y),
            ],  # Sağ, X
            [
                2 * (self.x * self.y + self.w * self.z),
                1 - 2 * (self.x**2 + self.z**2),
                2 * (self.y * self.z - self.w * self.x),
            ],  # Yukarı, Y
            [
                2 * (self.x * self.z - self.w * self.y),
                2 * (self.y * self.z + self.w * self.x),
                1 - 2 * (self.x**2 + self.y**2),
            ],
        ]  # Ön, Z

    def clone(self):
        return Quaternion(self.x, self.y, self.z, self.w)


class PID:
    def __init__(self, **kwargs):
        self.kp = kwargs.get("kp", 0)
        self.ki = kwargs.get("ki", 0)
        self.kd = kwargs.get("kd", 0)
        self.g = kwargs.get("g", 1)

        self.proportional = 0
        self.integral = 0
        self.differential = 0

        self.integral_min = -float("inf")
        self.integral_max = float("inf")

        self.set_point = kwargs.get("set_point", 0)
        self.output = 0
        self.error = 0
        self.previous_error = 0

        self.min_dt = 0.001

    def update(self, measured_value, dt):
        self.error = self.set_point - measured_value
        self.proportional = self.error
        self.integral = clamp(
            self.integral + self.error * dt, self.integral_min, self.integral_max
        )
        self.differential = (self.error - self.previous_error) / dt

        self.output = self.g * (
            self.proportional * self.kp
            + self.integral * self.ki
            + self.differential * self.kd
        )

        self.previous_error = self.error

        return self.output

    def set_integral_clamp(self, min, max):
        self.integral_min = min
        self.integral_max = max

    def reset(self):
        self.proportional = 0
        self.integral = 0
        self.differential = 0
        self.error = 0
        self.previous_error = 0
        self.output = 0
