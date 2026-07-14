
/// Represents the fixed-timestep clock for the simulation.
pub struct Clock { 
    /// The total number of physics steps that have been executed.
    tick: u64, 

    /// The duration of each fixed physics step, in seconds.
    fixed_dt: f64,

    /// The total simulated time elapsed, in seconds.
    elapsed_time: f64,

    /// Accumulated real-world time waiting to be consumed as fixed steps.
    accumulator: f64,

    /// Small time tolerance to avoid missing a step due to floating point error.
    /// Derived from fixed_dt * epsilon_factor.
    epsilon: f64,

    /// Maximum real-world frame time consumed per advance() call.
    /// Caps the accumulator to prevent the simulation spiral of death
    /// when a frame takes unusually long (e.g. debugger pause, tab switch).
    max_frame_time: f64,
}

impl Clock {

    /// Default maximum frame time in seconds.
    /// Caps the accumulator to prevent the spiral of death on long frames.
    pub const DEFAULT_MAX_FRAME_TIME: f64 = 0.25;

    /// Default epsilon as a fraction of fixed_dt.
    /// Prevents floating point error from causing missed steps.
    pub const DEFAULT_EPSILON_FACTOR: f64 = 0.01;

    /// Creates a new clock running at `hz` physics steps per second.
    pub fn new(hz: f64) -> Self {
        assert!(hz > 0.0, "Clock hz must be positive");
        let fixed_dt = 1.0 / hz;
        Self {
            tick: 0,
            fixed_dt,
            elapsed_time: 0.0,
            accumulator: 0.0,
            epsilon: fixed_dt * Self::DEFAULT_EPSILON_FACTOR,
            max_frame_time: Self::DEFAULT_MAX_FRAME_TIME,
        }
    }

    /// Total number of physics steps executed since creation.
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Total simulated time in seconds — sum of all fixed_dt steps taken.
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    /// Remaining real-world time not yet consumed as a physics step.
    /// Always in the range [0, fixed_dt).
    pub fn accumulator(&self) -> f64 {
        self.accumulator
    }

    /// Duration of each physics step in seconds.
    pub fn fixed_dt(&self) -> f64 {
        self.fixed_dt
    }

    /// Changes the physics step rate to `hz` steps per second.
    /// Recalculates epsilon using the default epsilon factor.
    pub fn set_fixed_dt(&mut self, hz: f64) {
        assert!(hz > 0.0, "Clock hz must be positive");
        self.fixed_dt = 1.0 / hz;
        self.epsilon  = self.fixed_dt * Self::DEFAULT_EPSILON_FACTOR;
    }

    /// Time tolerance used to avoid missing a step due to floating point error.
    /// Derived from fixed_dt * epsilon_factor.
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// Sets epsilon as a fraction of fixed_dt.
    /// A factor of 0.01 means steps trigger when accumulator >= fixed_dt * 0.99.
    pub fn set_epsilon(&mut self, factor: f64) {
        assert!(factor > 0.0, "Epsilon factor must be positive");
        self.epsilon = self.fixed_dt * factor;
    }

    /// Maximum real-world frame time consumed per advance() call, in seconds.
    pub fn max_frame_time(&self) -> f64 {
        self.max_frame_time
    }

    /// Sets the maximum frame time cap in seconds.
    pub fn set_max_frame_time(&mut self, max: f64) {
        assert!(max > 0.0, "Max frame time must be positive");
        self.max_frame_time = max;
    }

    /// Feeds real-world elapsed time and returns the number of physics steps to run.
    /// `frame_time` is capped at `max_frame_time` before being added to the accumulator.
    pub fn advance(&mut self, frame_time: f64) -> u32 {
        self.accumulator += frame_time.min(self.max_frame_time);

        let mut steps = 0u32;
        while self.accumulator >= (self.fixed_dt - self.epsilon) {
            self.accumulator  -= self.fixed_dt;
            self.elapsed_time += self.fixed_dt;
            self.tick         += 1;
            steps             += 1;
        }
        steps
    }

    /// Interpolation factor for render smoothing between physics ticks.
    /// Returns a value in [0.0, 1.0] representing how far the accumulator
    /// is through the current tick. Pass to lerp for smooth visual positions.
    pub fn alpha(&self) -> f64 {
        (self.accumulator / self.fixed_dt).clamp(0.0, 1.0)
    }
}



/********************/ 
/*      TESTS       */ 
/********************/ 
#[cfg(test)]
mod tests {
    use super::*;

     #[test]
    fn test_accumulator_cap() {
        let mut clock = Clock::new(100.0);  
        
        // Pass 10 seconds. It should cap at Clock::MAX_FRAME_TIME (e.g., 0.25)
        let steps = clock.advance(10.0);
        
        // At 100Hz, 0.25s is exactly 25 steps.
        // If it didn't cap, it would be 1000 steps.
        assert_eq!(steps, 25);
        assert_eq!(clock.tick(), 25);
    }

    #[test]
    fn test_partial_accumulation() {
        let mut clock = Clock::new(100.0);
        
        // 0.005 is half a tick.
        let steps = clock.advance(0.005);
        assert_eq!(steps, 0);
        
        // Another 0.005 completes the tick.
        let steps = clock.advance(0.005);
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_long_term_drift() {
        let mut clock = Clock::new(60.0);
        let dt = 1.0 / 60.0;
        
        // Simulate 10,000 steps
        for _ in 0..10000 {
            clock.advance(dt);
        }
        
        let expected = 10000.0 / 60.0;
        // Use a small epsilon for float comparison
        assert!((clock.elapsed_time() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_alpha_values() {
        let mut clock = Clock::new(100.0);  
        
        // Exactly half a tick
        clock.advance(0.005);
        assert_eq!(clock.accumulator(), 0.005);
        
        // Exactly a full tick - accumilator should reset to 0.0
        clock.advance(0.005);
        assert_eq!(clock.accumulator(), 0.0);
        
        // Slightly over one tick
        clock.advance(0.012);  
        assert_eq!(clock.accumulator(), 0.002);
    }

    #[test]
    fn test_weird_inputs() {
        let mut clock = Clock::new(60.0);
        
        // Zero time should do nothing
        assert_eq!(clock.advance(0.0), 0);
        
        // Negative time should be treated as 0 (due to .min() logic and loop bounds)
        assert_eq!(clock.advance(-1.0), 0);
        assert_eq!(clock.tick(), 0);
    }

    #[test]
    fn test_high_fps_low_dt() {
        let mut clock = Clock::new(60.0);  
        let mut total_steps = 0;
        
        // Simulate 100 frames at 200fps (0.005 per frame)
        for _ in 0..100 {
            total_steps += clock.advance(0.005);
        }
        
        // 100 * 0.005 = 0.5 seconds. 0.5 * 60Hz = 30 steps.
        assert_eq!(total_steps, 30);
        assert_eq!(clock.tick(), 30);
    }

    #[test]
    fn test_epsilon_boundary() {
        let hz = 100.0;
        let mut clock = Clock::new(hz);
        let dt = clock.fixed_dt();
        
        // Create a frame time that is slightly LESS than dt, 
        // but within the epsilon threshold.
        let just_under_dt = dt - (clock.epsilon * 0.5);
        
        let steps = clock.advance(just_under_dt);
        
        // It should trigger 1 step even though we didn't technically hit 0.01
        assert_eq!(steps, 1, "Epsilon should have triggered a step at {}", just_under_dt);
        
        // The accumulator should now be negative (or very close to -0.00005)
        // because we "overspent" our time.
        assert!(clock.accumulator() < 0.0); 
    }

    #[test]
    fn test_alpha_partial_progress() {
        let mut clock = Clock::new(100.0);  
        
        // Advance by 25% of a tick
        clock.advance(0.0025);
        assert_eq!(clock.alpha(), 0.25);
        
        // Advance by another 50%
        clock.advance(0.005);
        assert_eq!(clock.alpha(), 0.75);
    }


    #[test]
    fn test_alpha_reset_after_step() {
        let mut clock = Clock::new(100.0);  
        
        // Advance by 1.5 ticks
        clock.advance(0.015); 
        
        // 0.01 is consumed by the step. 0.005 remains.
        // 0.005 / 0.01 = 0.5
        assert!((clock.alpha() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_alpha_epsilon_not_negative() {
        let mut clock = Clock::new(100.0);  
        
        // Advance by exactly the threshold (0.0099)
        // This triggers a step. Accumulator becomes 0.0099 - 0.01 = -0.0001
        clock.advance(0.0099); 
        
        assert_eq!(clock.alpha(), 0.0);  
    }


    #[test]
    fn test_alpha_capped() {
        let mut clock = Clock::new(100.0); // dt = 0.01
        
        // Pass 10 seconds (way over MAX_FRAME_TIME of 0.25)
        clock.advance(10.0); 
        // 0.25 is exactly 25 steps. Accumulator should be 0.0.
        assert_eq!(clock.alpha(), 0.0);
    }



   
}
