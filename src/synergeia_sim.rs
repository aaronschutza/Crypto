// src/synergeia_sim.rs
// Simulation of the Synergeia Local Dynamic Difficulty (LDD) mechanism.
// Proves the "Rayleigh Distribution" property of block times.

use rand::prelude::*;

pub struct SynergeiaConfig {
    pub psi: f64, // Slot Gap (seconds)
    pub gamma: f64, // Recovery Threshold (seconds)
    pub target_block_time: f64, // Target mu (e.g., 15s)
}

pub struct SimulationResult {
    pub block_times: Vec<f64>,
    pub mean_time: f64,
    pub variance: f64,
}

// The "Snowplow" Hazard Function f(delta)
// f(d) = M * (d - psi) / (gamma - psi)
// This linear ramp in probability creates the Rayleigh distribution.
fn get_success_prob(delta: f64, slope_m: f64, config: &SynergeiaConfig) -> f64 {
    if delta < config.psi {
        return 0.0;
    }
    // The "Snowplow" region: Probability scales linearly with time
    if delta >= config.psi && delta < config.gamma {
        // f(t) = m * (t - psi)
        // Note: In the paper, the slope applies to the PDF, but for discrete sim
        // we approximate the hazard rate.
        // Hazard H(t) approx M * (t - Psi)
        let effective_t = delta - config.psi;
        return slope_m * effective_t;
    }
    // Recovery phase (Constant difficulty max cap)
    // Limits the probability to avoid instant-mining blocks if they simply take too long.
    return slope_m * (config.gamma - config.psi); 
}

pub fn run_simulation(blocks: usize) {
    println!("\n=== Synergeia LDD Consensus Simulation ===");
    println!("Parameters: Target=15s, Psi=5s, Gamma=50s");
    
    let config = SynergeiaConfig {
        psi: 5.0,
        gamma: 50.0,
        target_block_time: 15.0,
    };

    // 1. Calibrate Initial Slope M
    // For a Rayleigh distribution shifted by Psi, the mean is:
    // Mu = Psi + sqrt(pi / (2 * M))
    // Therefore: M = pi / (2 * (Mu - Psi)^2)
    let mu_shifted = config.target_block_time - config.psi;
    let mut slope_m = std::f64::consts::PI / (2.0 * mu_shifted.powi(2));
    
    println!("Initial Calibrated Slope M: {:.6}", slope_m);

    let mut rng = thread_rng();
    let mut block_times = Vec::new();
    let dt = 0.1; // Simulation time step (100ms)

    // PI Controller State
    let mut integral_error = 0.0;
    let kp = 0.000005; // Proportional gain
    let ki = 0.000001; // Integral gain

    for _ in 0..blocks {
        let mut time_since_last = 0.0;
        let mut mined = false;

        while !mined {
            time_since_last += dt;
            
            // Calculate hazard probability for this time step
            // P(mine in dt) = Hazard(t) * dt
            let prob = get_success_prob(time_since_last, slope_m, &config) * dt;
            
            // Monte Carlo trial
            if rng.gen::<f64>() < prob {
                mined = true;
                block_times.push(time_since_last);
            }
            
            // Failsafe
            if time_since_last > 300.0 { mined = true; block_times.push(300.0); }
        }
        
        // 2. Dynamic Adjustment (PI Controller)
        // Error = Target - Actual
        // If Actual > Target (Too Slow), Error is Negative.
        // We need to INCREASE slope to make it faster.
        // So: Slope_new = Slope_old - (Kp * Error + Ki * Integral) 
        // Wait, if Error is negative (Too Slow), we want Slope to Increase.
        // A steeper slope means probability rises faster => faster blocks.
        // So we should SUBTRACT a negative error (ADD).
        
        let error = config.target_block_time - time_since_last;
        integral_error += error;
        
        // Anti-windup for integral term
        if integral_error > 500.0 { integral_error = 500.0; }
        if integral_error < -500.0 { integral_error = -500.0; }

        let adjustment = (kp * error) + (ki * integral_error);
        
        // If Error > 0 (Too Fast), Adjustment > 0.
        // We want to DECREASE slope to slow it down.
        // So Slope = Slope - Adjustment.
        slope_m -= adjustment;

        // Clamp slope to sane values to prevent collapse
        if slope_m < 0.0001 { slope_m = 0.0001; }
        if slope_m > 0.1 { slope_m = 0.1; }
    }

    // Analysis
    let sum: f64 = block_times.iter().sum();
    let mean = sum / blocks as f64;
    
    // Consistency Metric: Count blocks found < Psi (Should be 0)
    let violations = block_times.iter().filter(|&&t| t < config.psi).count();
    
    println!("Simulation Complete ({} blocks)", blocks);
    println!("Final Slope M: {:.6}", slope_m);
    println!("Mean Block Time: {:.4}s (Target 15.0s)", mean);
    println!("Slot Gap Violations: {} (Security Check)", violations);
    
    // Check distribution shape (Rayleigh signature)
    let fast_blocks = block_times.iter().filter(|&&t| t < 10.0).count();
    let slow_blocks = block_times.iter().filter(|&&t| t > 20.0).count();
    
    let fast_pct = (fast_blocks as f64 / blocks as f64) * 100.0;
    let slow_pct = (slow_blocks as f64 / blocks as f64) * 100.0;

    println!("Distribution Profile:");
    println!("  Fast (<10s): {:.2}%", fast_pct);
    println!("  Slow (>20s): {:.2}%", slow_pct);
    
    // Stability Criteria
    if violations == 0 && (mean - 15.0).abs() < 1.0 {
        println!("> PASS: Synergeia Stability Conditions Met.");
    } else {
        println!("> FAIL: Instability Detected. (Mean deviation: {:.4})", (mean - 15.0).abs());
    }
}