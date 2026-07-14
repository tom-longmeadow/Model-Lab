/// Performance metrics captured during [`Simulation::simulate`].
///
/// These are written every frame and expose the wall-clock cost of the
/// simulation work so callers can detect performance problems:
///
/// - Is `step_time_ms > 1000 / hz`? The sim is spending more time per step
///   than its budget — it will fall further behind every frame (spiral of death).
/// - Is `steps_per_frame > 1` regularly? The sim is catching up after a slow frame.
/// - Is `steps_per_frame == 0` always? The frame rate is far above the sim rate —
///   normal, but `accumulator_ms` is consuming the slack.
/// - Is `substep_time_ms` close to `step_time_ms`? The cost is in the solver,
///   not the step bookkeeping.
#[derive(Clone, Copy, Debug, Default)]
pub struct SimMetrics {
    /// The configured simulation rate in Hz.
    pub hz: f64, 

    /// Wall-clock milliseconds to complete all substeps of a single fixed step.
    /// Zero if no steps ran this frame.
    pub step_time_ms: f64,

    pub storage_size: usize,

    // /// Average wall-clock milliseconds per substep.
    // /// Zero if no substeps ran this frame.
    // pub substep_time_ms: f64,

    // /// Number of fixed steps that ran during the last `simulate()` call.
    // /// 0 = frame too fast for sim rate, >1 = catching up after a slow frame.
    // pub steps_per_frame: u32,

    /// Banked time (ms) waiting to be consumed by the next fixed step.
    /// Should oscillate near zero; a steadily growing value means the sim
    /// cannot keep up with real time.
    pub accumulator_ms: f64,

    /// Total fixed steps executed since the simulation was created.
    pub total_ticks: u64,

   
}
