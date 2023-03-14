use std::{fs::{self, File}, io::BufWriter};

use estimation_distributions::*;

/*
 * - Monte carlo for each task is independant
 * - Generate `SAMPLE` entries for each task and store in Vec
 * - Add entries up for tasks for final distribution
 */

const INPUT_PATH: &str = "tasks.csv";
const OUTPUT_PATH_HISTOGRAM: &str = "histogram.csv";
const OUTPUT_PATH_SAMPLES: &str = "samples.csv";
const OUTPUT_PATH_STATS: &str = "stats.csv";
const SAMPLE_COUNT: usize = 1_000_000;
const BUCKET_COUNT: usize = 35;
const CHART_LINE_LENGTH: usize = 50;
const PERCENTILES_COUNT: usize = 11;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CSV
    let tasks = parse_tasks_from_csv(fs::read_to_string(INPUT_PATH)?.as_str())?;

    // Generate samples
    let total_task = Task {
        name: "Total".to_string(),
        min: 0.0,
        mode: 0.0,
        max: 0.0,
    };
    let sampled_tasks = run_monte_carlo(&tasks, SAMPLE_COUNT, &total_task);

    // Output samples to file
    let mut samples_buf_writer = BufWriter::new(File::create(OUTPUT_PATH_SAMPLES)?);
    write_samples_as_csv(&mut samples_buf_writer, &sampled_tasks)?;

    // Bucket samples
    let bucketed_samples = bucket_samples(&sampled_tasks[&sampled_tasks.len() - 1..], BUCKET_COUNT);

    // Draw histogram
    let msg = "Histogram of total values:";
    println!("{msg}\n{}", "=".repeat(msg.len()));
    write_histogram_as_ascii_art(
        &mut std::io::stdout(),
        &bucketed_samples.last().unwrap(),
        CHART_LINE_LENGTH,
    )?;

    // Write histogram data to CSV
    let mut histogram_buf_writer = BufWriter::new(File::create(OUTPUT_PATH_HISTOGRAM)?);
    write_histogram_as_csv(
        &mut histogram_buf_writer,
        &bucketed_samples.last().unwrap(),
    )?;

    // Calculate stats on samples
    let stats = calculate_stats(&sampled_tasks.last().unwrap(), PERCENTILES_COUNT)?;
    let mut stats_buf_writer = BufWriter::new(File::create(OUTPUT_PATH_STATS)?);
    stats.write_as_csv(&mut stats_buf_writer)?;
    stats.write_as_ascii(&mut std::io::stdout())?;

    Ok(())
}
