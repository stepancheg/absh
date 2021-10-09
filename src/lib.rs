pub use bars::plot;
pub use bars::plot_halves;
pub use bars::plot_halves_u64;
pub use bars::plot_u64;
pub use bars::PlotHighlight;
pub use duration::Duration;
pub use mem_usage::MemUsage;
pub use number::Number;
pub use numbers::Numbers;
pub use run_log::RunLog;
pub use stats::Stats;

pub mod ansi;
mod bars;
mod duration;
mod mem_usage;
mod number;
mod numbers;
mod run_log;
pub mod sh;
mod stats;
pub mod student;
