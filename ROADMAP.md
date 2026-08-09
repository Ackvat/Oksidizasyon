# oksidizasyon Roadmap

A learning-first rebuild of [ackmetton](documents/example_python/ackmetton/) in Rust.
Each phase is sized to teach one or two Rust ideas while producing a real piece of
the library. Order matters — every phase leans on the previous one.

Working agreement: steps are taken only when directed, kept as small as possible,
and the code is written by me (Ackvat) — Claude guides, reviews, and explains.

---

## Phase 0 — Solid ground

Fix the paper cuts in `src/base.rs` so the file compiles clean and clippy-quiet.

- [x] Remove or repair the dead `with_parent` (takes `self` by value, mutates, drops — a no-op).
- [x] Fix the broken test (references a variable that doesn't exist).
- [ ] `impl Default for Nodespace` (clippy: `new_without_default`).
- [x] Placeholder `SinkError` type so the `Sink` trait compiles (done properly in Phase 3).

**Teaches:** modules, `impl` blocks, what clippy expects of idiomatic code.

**Done when:** `cargo check` and `cargo clippy` pass with no warnings from `base.rs`.

---

## Phase 1 — Finish the tree *(ackmetton's `Base`)*

The index-arena scene tree: `Nodespace` owns `Vec<Option<Node>>`, nodes refer to
each other by `usize` id, deletion tombstones the slot (`None`) so ids stay stable.

- [x] `get_node` / `get_node_mut` — the `.get(id)?.as_ref()` chain.
- [x] `remove_node` — `Option::take`, detach parent/children with sequenced borrows.
- [x] `find_node(&str)` — iterate slots, skip tombstones, early return.
- [ ] `set_parent(child, Option<parent>)` — detach old, attach new, three short borrows.
- [ ] `add_node` taking an optional parent.

**Teaches:** borrow sequencing (one `&mut` at a time, copy `usize` ids out between
borrows), `Option` combinators (`?`, `take`, `as_ref`, `flatten`), lifetime elision,
`&str` vs `String` in signatures (borrow to read, own to store).

**Key insight to retain:** the borrow checker rejects self-referential structures;
indices instead of references is the standard Rust answer (slotmap, petgraph, every
scene graph). Raw pointers were tried and dangled — that experiment is documented
by the git history.

**Done when:** a test builds a 3-node tree, reparents one, removes one, and every
lookup behaves (including a stale id returning `None`).

---

## Phase 2 — The lifecycle trait *(ackmetton's `Module`)*

The open/close/run/stop lifecycle that `Module` gave every device class, as a trait.

- [ ] `trait Module` with `open`, `close`, `switch`, `run`, `stop` — default method
      bodies where behavior is generic, required methods where it isn't.
- [ ] One toy struct implementing it to prove the shape.

**Teaches:** traits as capability contracts (vs. Python inheritance), default
methods, the trait-objects (`&mut dyn Module`) vs. generics (`impl Module`) choice.

**Done when:** the toy struct can be driven through its lifecycle in a test via
`&mut dyn Module`.

---

## Phase 3 — Sinks *(ackmetton's `Logger` / `Printer`)*

Anything that accepts bytes: log buffers, UARTs, later SD cards and radios.

- [ ] `FeedLevel` enum (from `FEED_LEVELS`) — deriving `PartialOrd` so
      `level >= FeedLevel::Warn` works like the Python integer comparison did.
- [ ] A real `SinkError` enum (replacing the Phase 0 placeholder).
- [ ] Ring-buffer logger: fixed `[u8; N]` storage, head/tail, overwrite-oldest —
      `Discard_Oldest_Queue` reborn, no allocation after construction.
- [ ] Level-filtered write helper (the `Printer.debug/info/warn/error` surface).

**Teaches:** enums with ordering, `no_std`-friendly fixed buffers and index
wrapping, `Result` and custom error types, why panicking APIs are banned in
flight code.

**Done when:** writing more than N bytes wraps correctly (test proves oldest data
is overwritten, not panicked on).

---

## Phase 4 — Components meet nodes

The composition move that replaces ackmetton's inheritance chain
(`Base → Logged → Module → UART_Base`): a node *has* components instead of
*being* a subclass.

- [ ] `Node<C>` / `Nodespace<C>` generic over the component type (largely done).
- [ ] `attach` on both `Node` and `Nodespace` (by id).
- [ ] Application-side pattern documented: `enum Component { Log(Logger), Uart(UARTPort), … }`
      with `as_sink_mut()` / `as_update_mut()` capability dispatch.
- [ ] `tick(&mut Nodespace<C>, dt)` — walk all nodes, update every `Update` component.

**Teaches:** generics and monomorphization, `dyn` dispatch through enum match
arms, why the *library* defines traits and the *application* defines the enum
(open capability set, closed inventory).

**Key insight to retain:** the library owns the structure (tree, ids, trait
contracts); each robot's crate owns the inventory (which components exist).
This is `platform.py`'s job moved from runtime checks into the crate graph.

**Done when:** a node with two sinks attached receives one broadcast write on both.

---

## Phase 5 — utils.rs for real *(ackmetton's `utils.py`)*

- [ ] `TimeStamp` → dt tracker. Design question to settle: the library can't call
      `time.time()` in `no_std` — it receives timestamps and computes deltas, or
      takes a monotonic-clock trait the app implements.
- [ ] Generalize the Phase 3 ring buffer into `RingBuf<T, const N: usize>`.

**Teaches:** const generics (the embedded-Rust signature move), designing around
"the library has no clock, no heap growth, no OS".

---

## Phase 6 — The platform boundary *(ackmetton's `platform.py` + `DEVS`)*

Where `is_cpython()` runtime branching becomes compile-time structure.

- [ ] Decide the crate split: oksidizasyon = tree + maths + traits + utils;
      hardware impls (`serialport`/`rppal` for Pi 4, `rp-hal`/`embassy` for Pico)
      live in the app crates.
- [ ] Feature audit: what `std` vs `emd` actually gate today, what they should.
- [ ] `Uart` trait mirroring `UART_Base`'s surface (`write`, `read`, `read_all`,
      open/close via `Module`), with a std impl runnable on the dev machine.
- [ ] Per-target default consts (the `DEVS` pin tables) behind feature gates.

**Teaches:** cargo features, `#[cfg]`, workspaces, designing a trait so two
wildly different backends (pyserial-style fd vs. memory-mapped peripheral) both
fit behind it.

---

## Phase 7 — Maths audit + PID *(ackmetton's `mathlib.py`)*

Port the math — **not** the bugs. Known ackmetton defects to check `maths.rs` against:

| mathlib.py bug | Correct behavior |
|---|---|
| `Vector3.cross` returns a garbled scalar | Real cross product returning `Vector3`: `(y·bz − z·by, z·bx − x·bz, x·by − y·bx)` |
| `Vector3.__mult__` typo (never callable as `*`) | `impl Mul` |
| `median()` is actually the mean | Name it `mean()` |
| `unit()` is per-component sign | Name it `signum()`; `normal()` is the unit vector |
| `__eq__` compares magnitudes (`(3,4) == (5,0)`) | `#[derive(PartialEq)]` — component-wise |
| `PID.update` divides by unclamped `dt` (`min_dt` existed, unused) | `let dt = dt.max(MIN_DT);` skip D-term on first update |

- [ ] Audit existing `maths.rs` against the table.
- [ ] Port `PID` with the fixes, plus `clamp`, `map`, `deadzone`, `low_pass`.
- [ ] Tests that would have caught each Python bug.

**Teaches:** operator overloading (`impl Mul`, `impl Add`), `#[derive]`, tests as
bug documentation.

---

## Phase 8 — First integration

- [ ] A tiny binary (or example): build a nodespace, attach a logger sink and a
      fake sensor (`Update` impl producing synthetic data), tick the tree, watch
      output on stdout.

The "it flies on my desk" moment. Checkpoint afterwards: what does the Pico
target actually need next (allocator init, panic handler, defmt?) — that scoping
becomes Phase 9.

---

## Standing decisions

- **No raw pointers, no self-referential structs.** Ids (`usize`) are the only
  cross-node references. Tried, dangled, settled.
- **Tombstone deletion.** `Vec::remove` shifts indices and corrupts the tree;
  slots become `None` instead. Free-list reuse and generation counters are
  known future options if node churn appears.
- **Errors as values.** ackmetton returned `True/False/None`; here it's
  `Result`/`Option`, and the compiler makes callers look.
- **`children_count`-style duplicate state is banned** — one source of truth
  (`children.len()`).
- **Panics are for bugs, not conditions.** `.get()` over indexing, no `unwrap()`
  outside tests.
