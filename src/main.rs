use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;

/*
 * - Monte carlo for each task is independant
 * - Generate `SAMPLE` entries for each task and store in Vec
 * - Add entries up for tasks for final distribution
 */

const TASK_NAMES: [&'static str; 3] = [
    "Pull data from API",
    "Integration DB setup",
    "Transform data in integration DB and insert into Console",
];
const TASK_ESTIMATES_MIN: [f32; 3] = [1.5, 0.5, 2.0];
const TASK_ESTIMATES_MODE: [f32; 3] = [2.5, 1.0, 2.75];
const TASK_ESTIMATES_MAX: [f32; 3] = [5.0, 3.0, 4.0];
const SAMPLE_COUNT: usize = 100_000;

fn main() {
    let samples: Vec<Vec<f32>> = sample_task_estimates(
        &TASK_ESTIMATES_MIN,
        &TASK_ESTIMATES_MODE,
        &TASK_ESTIMATES_MAX,
        SAMPLE_COUNT,
    );

    write_samples_as_csv_to_stdout(&samples);
}

fn write_samples_as_csv_to_stdout(samples: &Vec<Vec<f32>>) {
    for column_header in (&["Sample"])
        .iter()
        .chain(TASK_NAMES.iter())
        .chain(&["Total"])
    {
        print!("{column_header}, ");
    }
    println!();
    for (sample_idx, sample) in samples.iter().enumerate() {
        print!("{sample_idx},");
        for estimate_idx in 0..TASK_NAMES.len() + 1 {
            let task_time = sample[estimate_idx];
            print!("{task_time},");
        }
        println!();
    }
}

fn sample_task_estimates(
    task_estimates_min: &[f32],
    task_estimates_mode: &[f32],
    task_estimates_max: &[f32],
    sample_count: usize,
) -> Vec<Vec<f32>> {
    let mut rng = thread_rng();
    let mut samples = Vec::new();
    for _sample_idx in 0..sample_count {
        let mut sample_output = vec![];
        for task_idx in 0..task_estimates_min.len() {
            let task_value = triangular_distribution_inv_cdf(
                rng.gen_range(0.0..=1.0),
                task_estimates_min[task_idx],
                task_estimates_mode[task_idx],
                task_estimates_max[task_idx],
            );
            sample_output.push(task_value);
            sample_output.push(sample_output.iter().sum());
        }
        samples.push(sample_output);
    }
    samples
}

fn triangular_distribution_inv_cdf(probability: f32, min: f32, mode: f32, max: f32) -> f32 {
    assert!(probability >= 0.0 && probability <= 1.0);
    let cdf_at_mode = (mode - min) / (max - min);
    if probability <= cdf_at_mode {
        min + f32::sqrt(probability * (mode - min) * (max - min))
    } else {
        max - f32::sqrt((max - min) * (max - mode) * (1.0 - probability))
    }
}

fn plot() {
    const SAMPLES: i32 = 1_000_000;
    const MIN: f32 = 0f32;
    const MODE: f32 = 20f32;
    const MAX: f32 = 50f32;

    let mut buckets = vec![0; 100];

    for i in 0..=SAMPLES {
        let t = 1.0 / SAMPLES as f32 * i as f32;
        let x = triangular_distribution_inv_cdf(t, MIN, MODE, MAX);
        buckets[x as usize] += 1;
        // println!("{t} => {x}");
    }
    for bucket in buckets {
        println!("{bucket}");
    }
}
