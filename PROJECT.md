# Oksidizasyon — Port Plan

A step-by-step plan for rewriting the Python library `ackmetton`
(`documents/example_python/ackmetton/`) as `oksidizasyon`, a `no_std` Rust library
targeting both a Raspberry Pi 4 (Linux, `std`) and a Raspberry Pi Pico (bare metal,
`no_std`), for use in a UAV and later robotics projects.

This document is the roadmap only. Nothing here is implemented yet beyond what is
noted as **done** in the inventory.

> **Revision 2.** The five open questions of revision 1 have been answered and are
> recorded in §6. The consequences are folded into §3 and §4: the scalar type is now
> concrete `f32`, the object tree is committed to rather than optional, the concurrency
> model is split async/blocking across the Pico's two cores, and Python is discarded
> entirely rather than kept behind an FFI boundary.

---

## 1. What ackmetton is

Six modules, roughly 1100 lines of Python, structured as a single-inheritance chain
with a runtime interpreter check at its core.

| Module | Contents |
| --- | --- |
| `platform.py` | `is_cpython()` / `is_micropython()` — runtime interpreter detection |
| `enums.py` | `SERIAL_ENUMS` (parity/stopbits/bytesize + a CPython→MicroPython parity map), `FEED_LEVELS` (numeric log levels), `DEVS` (per-board pin and bus constants) |
| `utils.py` | `Time_Stamp` (delta timing), `Discard_Oldest_Queue` (bounded, drops oldest on overflow), `List_Queue` (naive FIFO for MicroPython) |
| `mathlib.py` | Scalar helpers, `Vector2`, `Vector3`, `Basis`, `Quaternion`, `PID` |
| `base.py` | `Base` → `Logged` → `Module` → {`UART_Base`, `I2C_Base`}, plus `Logger`, `Printer`, `Log_Wrap`, `Logger_Silent` |

### The inheritance chain

```
Base                    name, parent, children[], is_cpython
 ├── Logger             stdlib logging → file handler
 ├── Printer            stdout with timestamp + level filter
 └── Logged             owns .log (Log_Wrap or Logger_Silent) and .print
      └── Module        open/running flags, Open/Close/Switch/Run/Stop
           ├── UART_Base   in/out queues, port config, Open_Serial/Write/Read/Read_All
           └── I2C_Base    bus handle, read/write byte · byte_data · block_data
```

### The two mechanisms that drive the whole design

1. **Runtime platform branching.** `is_cpython` is stored on every object at
   construction, and every platform-specific method branches on it. `I2C_Base` goes
   further and rebinds its own methods in `__init__` (`self.Read_Byte =
   self.cpython_Read_Byte`) to avoid paying for the branch per call.
2. **A parent/children object tree.** `Base` maintains a bidirectional graph so any
   object can be attached to, found under, or released from another. Per §6.1 this is a
   load-bearing feature, not incidental: it is how one body of code is meant to describe
   several different machines.

Mechanism 1 disappears into the type system. Mechanism 2 has to be rebuilt, because
Rust rejects its shape outright.

---

## 2. Inventory: Python → Rust

| ackmetton | oksidizasyon | Status |
| --- | --- | --- |
| `platform.is_cpython/is_micropython` | Cargo features + `#[cfg]` (compile time) | Phase 0 |
| `enums.FEED_LEVELS` | `base::Level` enum | **done** |
| `enums.SERIAL_ENUMS` | `comm::serial::{Parity, StopBits, DataBits}` | Phase 7 |
| `enums.DEVS` | `device::{rpi, pico}` const modules | Phase 9 |
| `utils.Time_Stamp` | `time::{Clock, TimeStamp}` | Phase 4 |
| `utils.Discard_Oldest_Queue` | `buffer::RingBuffer<T, N>` | Phase 5 |
| `utils.List_Queue` | dropped — `heapless::Deque` covers it | Phase 5 |
| `mathlib.clamp` | `maths::clamp` | **done**, drops generic in Phase 1 |
| `mathlib.map/deadzone` | `maths::` free functions | Phase 1 |
| `mathlib.low_pass/amplitude_impedance` | `filter` module | Phase 1 |
| `mathlib.sin/cos/acos` | `maths::scalar` (thin `libm` wrappers) | Phase 1 |
| `maths::Float` trait | **deleted** — superseded by concrete `f32` (§3.2) | Phase 1 |
| `mathlib.Vector2/Vector3` | `maths::Vector2/Vector3` (non-generic) | done as generic — Phase 1 rewrites |
| `mathlib.Basis` | `maths::Basis` | stub only — Phase 1 |
| `mathlib.Quaternion` | `maths::Quaternion` (non-generic) | done as generic — Phase 1 rewrites |
| `mathlib.PID` | `maths::PID` (non-generic) | done as generic — Phase 1 rewrites |
| `base.Base` | `base::Base` + `base::Object` trait | partially done — Phase 2 |
| `base.Base` parent/children | `node::{NodeId, Tree}` arena | Phase 2 |
| `base.Logger` | `base::Logger<S: Sink>` | **done** |
| `base.Log_Wrap` | the `source: &str` argument to `Logger::log` | **done** |
| `base.Logger_Silent` | `base::NopSink` | **done** |
| `base.Printer` | `sink::ConsoleSink` (std) / `defmt` (Pico) | Phase 3 |
| `base.Logged` | `base::Object` + logger passed per call | **done** |
| `base.Module` | `module::{Module, State}` | Phase 6 |
| `base.UART_Base` | `comm::uart::Uart<T>` over `embedded-io` | Phase 7 |
| `base.I2C_Base` | `comm::i2c::Bus<T>` over `embedded-hal` | Phase 7 |
| — | async variants of both | Phase 8 |

Note the status shift on the math types. They are written and working, but generic over
`T: Float`; §3.2 removes the generic, so Phase 1 is a rewrite of existing code rather
than an extension of it.

---

## 3. Translation decisions

All resolved. §6 records the answers these follow from.

### 3.1 Platform selection moves from runtime to compile time

`is_cpython()` becomes Cargo features. Nothing in the library branches on the platform
at runtime; the branch is resolved when the binary is built.

```toml
[features]
default = ["std"]
std   = ["alloc"]   # Pi 4: filesystem, console, threads
alloc = []          # heap without an OS — used only by the owned form of the tree
async = []          # embedded-hal-async transports (Phase 8)
# no features: Pico — static allocation only
```

Since Python is being discarded wholesale (§6.5), both boards run this crate directly
and there is no FFI surface to design around.

This is strictly better than the Python design: `I2C_Base` rebinds methods in
`__init__` specifically to dodge the per-call branch, and `#[cfg]` gives that for free
with no indirection.

### 3.2 One scalar type: `f32`

The `Float` trait and its two impls are deleted. Every math type becomes concrete `f32`.

The trait was written so a Pi 4 build could use `f64` and a Pico build `f32`. That
flexibility is not worth its cost:

- **Nothing in the flight math needs `f64`.** Attitude, rates, PID state and motor mixing
  all live in ranges where `f32`'s ~7 significant digits are ample.
- **The Pico has no FPU.** On RP2040's Cortex-M0+, *both* types are software-emulated,
  but `f32` uses the fast ROM-resident routines and `f64` is several times slower. On an
  RP2350 (Pico 2) the Cortex-M33 FPU is **single-precision only**, so `f64` gets no
  hardware help there either.
- **The generic costs readability on every line.** `T::from(0)`, `T::from(2)`,
  `T::from_f64(0.001)` and the `From<i16>` bound exist purely to write numeric literals.
  All of it disappears: `0.0`, `2.0`, `0.001`.

**The one real exception is global position.** Latitude/longitude in degrees genuinely
does not fit `f32` — at magnitude 180, one ULP is about 1.5e-5 degrees, roughly 1.7 m of
ground error, which is unusable for navigation. The standard workaround, used by MAVLink,
PX4 and ArduPilot alike, is to never store position as a float at all:

- Global position as `i32` in units of 1e-7 degrees (≈1.1 cm resolution, exact).
- Convert once to a local NED tangent-plane frame relative to a home point, and do all
  `f32` math there, where coordinates are metres from home and precision is abundant.

Altitude, velocity, and everything downstream stay `f32`.

To keep the door open, the `libm` calls are wrapped in a small `maths::scalar` module
rather than being sprinkled through the types. Changing scalar precision later then means
editing one file, not three hundred call sites. The `F32Vector3` / `F64Vector3` aliases
go away; `Vector3` simply *is* the `f32` one.

### 3.3 The object tree is committed to, backed by an arena

Per §6.1 the hierarchy stays. Two distinct needs hide inside that answer, and they want
different mechanisms:

| Need | Mechanism |
| --- | --- |
| One body of code describing several similar machines | Traits + generics — `Object`, `Module`, and the driver traits above them |
| Runtime composition: attach, find, release, traverse | The arena tree |

Both get built. The traits are the reuse story; the tree is what lets a specific airframe
be assembled and walked at runtime.

`Base.Add_Child` creates a reference cycle — parent holds child, child holds parent —
which Rust rejects outright. The options:

| Approach | Verdict |
| --- | --- |
| `Rc<RefCell<Node>>` | Needs `alloc`, leaks on cycles, runtime borrow panics. **No.** |
| Arena + integer IDs | Nodes in one flat array, links are indices. `no_std`-clean via `heapless::Vec`, no cycles, no borrow panics. **Yes.** |

The arena stores *links*, and the objects themselves are borrowed into it:

```
Node<'a> { object: &'a mut dyn Object, parent: Option<NodeId>, first_child: Option<NodeId>, next_sibling: Option<NodeId> }
Tree<'a, const N: usize> { nodes: heapless::Vec<Node<'a>, N> }
```

Consequences worth knowing before Phase 2 starts:

- Objects live in the application (statics or the main stack frame) and the tree holds a
  unique borrow of each. After registration they are reachable *only* through the tree.
  That is the price of no-allocation dynamic composition, and it is the normal embedded
  pattern.
- Sibling links rather than a per-node child `Vec` keeps every node fixed-size and the
  whole tree one flat allocation.
- `NodeId` should be a generational index (`{ index: u16, generation: u16 }`) so a stale
  ID from a released node cannot silently address whatever took its slot.
- `Object` **must be object-safe** for `&mut dyn Object` to work. This settles the open
  design point from revision 1: no generic methods on `Object`, so `report` takes
  `&mut dyn Sink`, not `impl Sink`.
- With `alloc` (Pi 4), an owned `Box<dyn Object>` variant can be offered as a
  convenience. Same API, no lifetime parameter.

### 3.4 Concurrency: async on core 0, blocking on core 1

Per §6.3, the Pico runs both models simultaneously — an Embassy executor on core 0 and a
plain blocking loop on core 1. This is a supported RP2040 pattern (`embassy-rp` with
`spawn_core1`) and it is a sound split:

| Core | Model | Work |
| --- | --- | --- |
| 1 | Blocking, fixed-rate | Attitude estimation, PID, mixer, motor output — deterministic timing matters more than throughput |
| 0 | Async | Telemetry, GPS NMEA parsing, companion-computer link, logging drain — I/O-bound and bursty |

What this costs, stated plainly: **the transport layer has to exist twice.**
`embedded-hal` and `embedded-hal-async` are separate trait sets, and Rust has no way to
be generic over "blocking or async" without either duplicating each method or generating
both from a macro. This is real work and it is why the split gets its own phase (8)
rather than being folded into Phase 7.

Three rules that follow, and they shape everything from Phase 5 onward:

1. **The core library stays I/O-free.** `maths`, `filter`, `base`, `node`, `module`,
   `time` and `buffer` touch no peripheral and are therefore agnostic by construction.
   Only `comm` and the drivers above it face the split.
2. **Peripherals are owned by exactly one core.** No sharing a bus across the FIFO
   boundary. Assign at build time.
3. **Cross-core data moves through one explicit channel**, not shared mutable state —
   `embassy-sync` with `CriticalSectionRawMutex`, or the raw SIO FIFO. This lands in
   Phase 5 alongside the buffers.

Blocking is built first. Async is added on top without disturbing it.

### 3.5 Errors become `Result`, not `bool`/`None`

`Open_Serial` returns `True`/`False` and swallows the exception into a log line;
`Read` returns `None` on both "no data" and "cannot communicate". In Rust:

- `Result<T, Error>` where the failure is actionable.
- `Option<T>` only where absence is normal and not an error (no byte ready yet).
- One crate-level `Error` enum, `#[non_exhaustive]`, no `Box<dyn Error>` (needs alloc).

### 3.6 State machines get types, not loose booleans

`Module` carries `open` and `running` as independent bools, which admits the nonsense
state `running && !open`. Replace with `enum State { Closed, Open, Running }`.

Typestate (`Module<Closed>` → `Module<Open>`) is now **ruled out** rather than deferred:
it is incompatible with `&mut dyn Object` in the tree, since each state would be a
distinct type. The runtime enum is the right call here.

### 3.7 Object names: `&'static str` plus an instance index

Per §6.4, names exist so a log line carries its source automatically — `[imu] ...`
without the call site repeating it. That is already how `Logger::log(level, source, args)`
works.

`&'static str` covers this, with one addition for the "four identical motors" case that
§6.1's multi-machine goal makes routine:

```
Base { name: &'static str, instance: Option<u8> }   // renders as "motor" or "motor[2]"
```

No formatting, no allocation, no `heapless::String`. If a genuinely dynamic name is ever
needed, `heapless::String<16>` behind a feature is the escape hatch — but the instance
index is expected to cover it.

### 3.8 Transports build on `embedded-hal`, not on our own abstraction

`UART_Base` and `I2C_Base` exist to paper over `pyserial`/`smbus2` vs `machine.UART`/
`machine.I2C`. Rust already solved this: `embedded-hal` 1.0 defines the peripheral
traits, and every platform ships an implementation.

| | Pi 4 | Pico |
| --- | --- | --- |
| I2C | `linux-embedded-hal` | `rp2040-hal` / `embassy-rp` |
| Serial | `serialport` behind `embedded-io` | `rp2040-hal` / `embassy-rp` |

So `comm::i2c::Bus<T>` is generic over `T: embedded_hal::i2c::I2c` and contains **zero**
platform code. This deletes the largest and most duplicated part of `base.py` — roughly
200 lines of paired `cpython_*`/`micropython_*` methods collapse into one generic impl.

### 3.9 Naming

`Add_Child` → `add_child`, `Open_Serial` → `open`, `GetBasis` → `basis()`. Standard Rust
casing throughout; getters drop the `get_` prefix per API guidelines.

### 3.10 Time has no source in `core`

`Time_Stamp` calls `time.time()`. `core` has no clock at all. The library defines a
`Clock` trait and the application supplies it — same pattern as `Sink`.

---

## 4. Roadmap

Each phase is independently useful and leaves the crate compiling. Phases 1–6 have no
hardware dependency and can be finished on the desktop.

### Phase 0 — Crate foundation
**Files:** `Cargo.toml`, `src/lib.rs`, `src/error.rs`

- Add the feature flags from §3.1.
- `#![no_std]` with `#[cfg(feature = "std")] extern crate std;`.
- Crate-level `Error` enum and `pub type Result<T> = core::result::Result<T, Error>`.
- Fix the two half-written structs at the bottom of `base.rs` (`UARTBasic`, `I2CBasic`) —
  they are Phase 7 work and currently break the parse.
- Declare the module tree so later phases only fill it in:
  `base`, `node`, `maths`, `filter`, `time`, `buffer`, `module`, `comm`, `device`.

**Done when:** the crate builds for both `x86_64-unknown-linux-gnu` and
`thumbv6m-none-eabi` with `--no-default-features`.

### Phase 1 — De-genericize and finish the math module
**Files:** `src/maths.rs`, new `src/filter.rs`

The largest single edit in the plan, and it should happen first because everything
downstream is typed in terms of these.

1. **Remove the generic.** Delete `trait Float` and both impls. `Vector2`, `Vector3`,
   `Basis`, `Quaternion`, `PID` become concrete `f32` types. Drop the `F32*`/`F64*`
   aliases. Every `T::from(0)` becomes `0.0`.
2. **Add `maths::scalar`** — thin `sinf`/`cosf`/`asinf`/`atan2f`/`sqrtf`/`fabsf`/
   `copysignf` wrappers over `libm`, so the precision decision is contained in one file.
3. **Close the gaps** against `mathlib.py`: `map`, `deadzone`, `median`, `absmedian`,
   and scalar `Mul`/`Div` by `f32` alongside the existing component-wise impls (Python's
   `Quaternion.__mul__` is the scalar form; both are wanted).
4. **`filter.rs`:** `low_pass` and `amplitude_impedance`, sited next to the future
   complementary filter rather than in `maths`.
5. **`Basis`:** currently a bare struct whose generic parameter shadows `Vector3`. With
   the generic gone it becomes a plain three-vector struct. Give it `identity()`,
   `transpose()`, `mul_vector()`, `from_quaternion()`, and have `Quaternion::get_basis`
   and `get_rotation_matrix` share one implementation instead of duplicating the nine
   terms.
6. **Fix:** `Quaternion::new_from_axis_angle` must normalize its axis, as Python does.
7. **Skip:** `__eq__`/`__lt__` by magnitude. Two different vectors of equal length are
   not equal; the derived component-wise `PartialEq` is correct.
8. Add `#[cfg(test)]` unit tests — desktop-only, no effect on the Pico build.

**Done when:** `maths.rs` has no type parameters, every `mathlib.py` capability worth
keeping has an equivalent, and tests cover quaternion↔euler round-trips and the
gimbal-lock branch.

### Phase 2 — Object model and the tree
**Files:** `src/base.rs`, new `src/node.rs`

Now a headline phase rather than a maybe, per §6.1.

- Move `report` into `Object` as a default method so subtypes inherit it — inherent
  methods do not inherit, trait defaults do.
- Take `&mut dyn Sink` in that signature, keeping `Object` object-safe (§3.3).
- Add `instance: Option<u8>` to `Base` and render it in log sources (§3.7).
- `node.rs`: the arena of §3.3 — generational `NodeId`, `Node<'a>` with parent/
  first-child/next-sibling links, `Tree<'a, N>` over `heapless::Vec`.
- Tree operations mapping the Python API: `add_child`, `find_child` (by name, and by
  name+instance), `set_parent`, `remove_child`, `release`, plus a depth-first iterator
  the Python never had.
- `find_child` should dedupe on insert — the Python lets one object be attached twice
  (§5).

**Done when:** a mock airframe of a dozen objects can be assembled, traversed, searched
and released, with tests, on the desktop.

### Phase 3 — Diagnostics
**Files:** `src/base.rs`, new `src/sink.rs`

The `Logger`/`Sink`/`Level` core is done. What remains is the sinks.

- `ConsoleSink` (`std`): replaces `Printer` — timestamp, level, message to stdout.
- `FileSink` (`std`): replaces `Logger`'s `FileHandler` — creates the log directory,
  opens `{dir}/{name}.log`, line-buffered.
- `TeeSink<A, B>` so console and file run together, as `logging` handlers do.
- Evaluate `defmt` for the Pico rather than a text sink: it ships format strings in the
  ELF and sends only IDs over the wire, which matters at a 200 Hz loop rate.
- Note the concurrency constraint from §3.4: a sink shared between cores needs a mutex,
  or — better — core 1 pushes records into a queue that core 0 drains and writes. Decide
  this here, since it affects the sink signature.
- Wire `Clock` (Phase 4) in for timestamps once it exists.

**Done when:** a Pi 4 binary writes a formatted log file, and the same code with
`NopSink` compiles to nothing on the Pico.

### Phase 4 — Time
**Files:** `src/time.rs`

- `trait Clock { fn now_micros(&self) -> u64; }` — application-supplied, as with `Sink`.
- `TimeStamp` on top: `start`, `end`, `delta`, matching `Time_Stamp`'s role but in
  integer microseconds rather than a float scaled by `unit_mask`.
- `Instant`/`Duration` newtypes, or adopt `fugit` — decide when writing it.
- Convert to `f32` seconds only at the boundary where `PID::update(dt)` wants them.
  A `u64` microsecond counter is exact for ~584,000 years; an `f32` seconds counter
  loses millisecond resolution after about four hours of uptime, so the integer form is
  the one that gets stored.

**Done when:** a loop can measure its own period on both targets.

### Phase 5 — Buffers, queues, and the cross-core channel
**Files:** `src/buffer.rs`, `src/channel.rs`

- `RingBuffer<T, const N: usize>` — the `Discard_Oldest_Queue` behaviour, which is
  exactly right for sensor data where the freshest sample wins. Fixed capacity, no alloc.
- For producer/consumer across an interrupt boundary, use `heapless::spsc::Queue`
  directly rather than wrapping it.
- **New, from §3.4:** the core-0/core-1 channel. Evaluate `embassy-sync` with
  `CriticalSectionRawMutex` against the raw SIO FIFO. Needed before Phase 8, and needed
  by Phase 3 if logging crosses cores.
- `List_Queue` is dropped; it existed only because MicroPython lacks `queue`.

**Done when:** `RingBuffer` is tested for the overflow-discards-oldest path and the
channel choice is settled.

### Phase 6 — Module lifecycle
**Files:** `src/module.rs`

- `enum State { Closed, Open, Running }` per §3.6.
- `trait Module: Object` with `open`/`close`/`toggle`/`run`/`stop`, default bodies
  driving a `State` reached through an accessor — the same pattern as `Object`.
- Must stay object-safe, so modules can sit in the Phase 2 tree.

**Done when:** a dummy module can be driven through the full state cycle from inside the
tree.

### Phase 7 — Transports, blocking
**Files:** `src/comm/mod.rs`, `src/comm/uart.rs`, `src/comm/i2c.rs`, `src/comm/serial.rs`

- `Uart<T>` generic over `embedded_io::{Read, Write}`.
- `Bus<T>` generic over `embedded_hal::i2c::I2c`. The twelve paired `cpython_*`/
  `micropython_*` methods collapse to six: `read_byte`, `read_register`, `read_block`,
  `write_byte`, `write_register`, `write_block`.
- Config types replacing `SERIAL_ENUMS`: `Parity`, `StopBits`, `DataBits` as enums. The
  CPython→MicroPython parity map disappears — each HAL converts its own.
- Port `Write`, `Read`, `Read_Line`, `Read_All`. `Read_All` is where the Python branches
  on `in_waiting` vs `any()`; `embedded-io`'s `ReadReady` covers both.
- `Read_Line` takes a caller-supplied `&mut [u8]` — no allocation for the line.
- The in/out queues become Phase 5 buffers, and stay optional.
- All return `Result`, never `None`/`True`.

**Done when:** a Pi 4 build talks to a loopback serial device and reads one real sensor
(MPU-6050 or similar) over I2C.

### Phase 8 — Transports, async, and the dual-core split
**Files:** `src/comm/*` (async variants), plus a Pico example binary

- Mirror Phase 7 over `embedded-hal-async` and `embedded-io-async`, behind
  `feature = "async"`.
- Decide the duplication strategy first: hand-written parallel methods, or macro-generated
  from one source (the `maybe-async` pattern). Hand-written is clearer for six methods;
  reconsider if the driver count grows.
- Stand up the split from §3.4 in a real example: Embassy executor on core 0, fixed-rate
  blocking control loop on core 1, one channel between them.
- Document which peripheral belongs to which core, and enforce it by moving the handles.

**Done when:** a Pico binary runs a control loop on core 1 while core 0 services a UART
asynchronously, with data crossing once through the Phase 5 channel.

### Phase 9 — Device constants
**Files:** `src/device/mod.rs`, `src/device/rpi.rs`, `src/device/pico.rs`

- `DEVS` becomes `const` items in per-board modules, gated by `#[cfg(feature)]`.
- Pin numbers stay plain `u8`; the HAL owns the real pin types.

**Done when:** no magic pin numbers remain in `comm`.

### Phase 10 — Beyond parity

Where the Rust version starts exceeding `ackmetton`. With Python discarded (§6.5) this
is the whole flight stack, not an optional extra.

- Sensor drivers (IMU, barometer, GPS, magnetometer) built on Phase 7.
- Global position handling per §3.2: `i32` 1e-7-degree storage, local NED conversion.
- Attitude estimation — complementary filter first, then Mahony/Madgwick, feeding the
  existing `Quaternion`.
- Mixer / motor output stage on top of `PID`.
- SIL harness on the desktop: the same control code, fed simulated sensor data through
  the same traits. Cheap now, since nothing in the core library touches hardware.
- A second airframe or robot assembled from the same crate — the real test of whether
  §6.1's reuse goal was met.

---

## 5. Bugs in the Python — reference only

Since Python is being discarded entirely (§6.5), these need no upstream fix. They are
recorded so the port does not faithfully reproduce them.

| Location | Problem |
| --- | --- |
| `mathlib.py:209` | `Vector3.cross` is mathematically wrong — it returns a scalar built from a garbled expression instead of a `Vector3`. The Rust version is already correct. |
| `base.py:152–188` | `Printer`'s level test is inverted: `self.printer_level >= FEED_LEVELS.DEBUG`. With the default level `DEBUG`, `error()` never prints. It should compare the record's level against the threshold, not the reverse. `Logger::log` already has this the right way round. |
| `mathlib.py:425–433` | `PID.min_dt` is defined and never used, so `update` divides by a caller-supplied `dt` that may be zero. The Rust `PID` already guards this. |
| `mathlib.py:146` | `Vector3.__mult__` is a typo for `__mul__`, so `Vector3 * Vector3` raises `TypeError`. |
| `base.py:71–94` | On MicroPython, `Logger.__init__` prints a warning and leaves `self.logger` unset — any later `debug()` raises `AttributeError`. |
| `base.py:403–406` | `I2C_Base` defines `sda`/`scl`/`freq` only on the MicroPython branch; the CPython branch leaves them missing. |
| `utils.py:3` | `from queue import Queue` is unconditional, but MicroPython has no `queue` module — importing `utils` there fails, which is precisely what `List_Queue` exists to avoid. |
| `mathlib.py:13` | `map` shadows the builtin within the module. |
| `base.py:29–46` | `Add_Child`/`Set_Parent` do not dedupe, so an object can be listed twice under one parent. Phase 2 fixes this. |
| `mathlib.py:76` | `__eq__` compares magnitudes, so `Vector2(1,0) == Vector2(0,1)` is `True`. Also defined without `__hash__`, making the type unhashable. |

---

## 6. Resolved decisions

The revision-1 open questions and their answers.

**6.1 — Is the parent/children tree needed? → Yes.**
The library will serve many robotics projects, so it needs a robust way to describe
similar machines with the same code. Consequence: the tree is committed to (§3.3),
`Object` must stay object-safe, and typestate modules are ruled out (§3.6). The answer
also implies a second, separable requirement — reuse across machines — which traits and
generics serve, not the tree; both are built.

**6.2 — `f32` or generic? → `f32` only, if `f64` is genuinely unnecessary.**
It is, with one exception. Attitude and control math are comfortable in `f32`, the Pico
has no double-precision hardware on either RP2040 or RP2350, and the generic costs
readability everywhere. The exception is global position, which needs `i32` 1e-7-degree
storage and a local NED frame — the standard workaround, not a compromise. See §3.2.

**6.3 — Async or blocking? → Both, split across the Pico's two cores.**
Async executor on core 0, blocking control loop on core 1. Supported by `embassy-rp`,
and the right split for the workload. Cost: the transport layer exists twice, which is
why it is Phases 7 and 8 rather than one. See §3.4.

**6.4 — Runtime-derived names? → Names exist to label log lines automatically.**
`&'static str` plus an `Option<u8>` instance index covers this without allocation. See
§3.7.

**6.5 — Does Python stay on the Pi 4? → No, the whole stack becomes Rust.**
No FFI or PyO3 surface to design. Phase 10 is the real deliverable, not a bonus, and the
`ackmetton` bugs in §5 need no upstream fix.

---

## 7. Still open

New questions raised by the answers above.

1. **RP2040 or RP2350?** The Pico 2's Cortex-M33 has a single-precision FPU; the
   original Pico has none. Both are fine under the `f32` decision, but the performance
   budget for the control loop differs by roughly an order of magnitude on float-heavy
   code.
2. **Tree capacity and ownership form.** `Tree<'a, N>` needs a concrete `N` per
   application. Is the borrowed form (§3.3) sufficient, or is the `alloc`-backed owned
   form wanted on the Pi 4 from the start?
3. **Is the tree built at runtime from configuration**, or assembled in code at startup?
   If the latter, `find_child`-by-name matters less than the traversal iterator, and the
   generational `NodeId` may be more machinery than needed.
4. **Which side drains the log?** If core 1 logs during the control loop, records must
   cross to core 0 rather than blocking on a write. Decide in Phase 3, not Phase 8.
5. **Control loop rate.** Sets the timing budget everything else is measured against, and
   determines whether `defmt` is required on the Pico or merely nice.
