# oksidizasyon

A `no_std` math library (vectors, quaternions, bases) built on `libm`.
It will be used with another Rust program as a library of tools and object.
Currently, it is planned to use it in a Rust program to be compiled for both a Raspberry Pi 4 and a Raspberry Pi Pico, in which they will also be used in a Unmanned Aerial Vehicle (UAV) project. Later on, the library will be built on and on to be a fitting one to be used in many other robotics projects.

## Working notes

- **Do not run tests, builds, or compiles** (`cargo build`, `cargo test`, `cargo check`,
  `cargo clippy`, ...) unless explicitly asked to. Write the code and stop there.

- **If asked about how to a such action, rather than for you to do it, do not execute any commands but tell the user how to do it**, for example, if the user asks you "How can I fix the issue inside the main function?", you will only them how to fix the problem rather than taking executive actions to fix it yourself.

- Do not make changes to files unless explicitly told to do so.

- Disregard hidden folders, the might contain unwanted (deprecated) features that might cause confusion.