use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Base {
    pub name: &'static str,
}

impl Base {
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }
}

pub trait Object {
    fn base(&self) -> &Base;
    fn base_mut(&mut self) -> &mut Base;

    fn name(&self) -> &'static str {
        self.base().name
    }

    fn set_name(&mut self, name: &'static str) {
        self.base_mut().name = name;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info  => "INFO",
            Level::Warn  => "WARN",
            Level::Error => "ERROR",
        }
    }
}

pub trait Sink {
    fn record(&mut self, level: Level, source: &str, args: fmt::Arguments);

    // Called at a safe point, not from the control loop, for sinks that buffer.
    fn flush(&mut self) {}
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct NopSink;

impl Sink for NopSink {
    fn record(&mut self, _level: Level, _source: &str, _args: fmt::Arguments) {}
}

pub struct FmtSink<W: fmt::Write> {
    pub writer: W,
}

impl<W: fmt::Write> FmtSink<W> {
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: fmt::Write> Sink for FmtSink<W> {
    fn record(&mut self, level: Level, source: &str, args: fmt::Arguments) {
        // Errors are dropped on purpose, see `Sink`.
        let _ = writeln!(self.writer, "[{}] {}: {}", level.as_str(), source, args);
    }
}

#[derive(Debug, Clone)]
pub struct Logger<S: Sink> {
    pub sink: S,
    pub min_level: Level,
    pub enabled: bool,
}

impl<S: Sink> Logger<S> {
    pub const fn new(sink: S) -> Self {
        Self { sink, min_level: Level::Info, enabled: true }
    }

    pub const fn with_min_level(mut self, min_level: Level) -> Self {
        self.min_level = min_level;
        self
    }

    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn log(&mut self, level: Level, source: &str, args: fmt::Arguments) {
        if self.enabled && level >= self.min_level {
            self.sink.record(level, source, args);
        }
    }

    pub fn flush(&mut self) {
        self.sink.flush();
    }
}

impl Default for Logger<NopSink> {
    fn default() -> Self {
        Self::new(NopSink)
    }
}

#[macro_export]
macro_rules! log {
    ($logger:expr, $level:expr, $source:expr, $($arg:tt)*) => {
        $logger.log($level, $source, core::format_args!($($arg)*))
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Basic {
    base: Base,
}

impl Basic {
    pub const fn new(name: &'static str) -> Self {
        Self { base: Base::new(name) }
    }

    pub fn report<S: Sink>(&self, logger: &mut Logger<S>, level: Level, args: fmt::Arguments) {
        logger.log(level, self.base.name, args);
    }
}

impl Object for Basic {
    fn base(&self) -> &Base {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
}

pub struct UARTBasic {
}

pub struct I2CBasic {
}