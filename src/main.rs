use rand::prelude::*;
use std::{
    fs::{self, File},
    io::Write,
};

/*
 * - Monte carlo for each task is independant
 * - Generate `SAMPLE` entries for each task and store in Vec
 * - Add entries up for tasks for final distribution
 */

const INPUT_PATH: &str = "tasks.csv";
const OUTPUT_PATH_HISTOGRAM: &str = "histogram.csv";
const OUTPUT_PATH_SAMPLES: &str = "samples.csv";
const OUTPUT_PATH_STATS: &str = "stats.csv";
const SAMPLE_COUNT: usize = 10_000;
const BUCKET_COUNT: usize = 50;
const CHART_LINE_LENGTH: usize = 50;
const PERCENTILES_COUNT: usize = 3;

fn main() -> std::io::Result<()> {
    let total_task = Task {
        name: "Total".to_string(),
        min: 0.0,
        mode: 0.0,
        max: 0.0,
    };
    // Parse CSV
    let mut tasks = parse_tasks_from_csv(fs::read_to_string(INPUT_PATH)?.as_str()).unwrap();

    // Generate samples
    let task_samples: Vec<TaskSamples> = sample_task_estimates(&tasks, SAMPLE_COUNT);
    tasks.push(total_task);

    // Output samples to file
    let mut sample_output_file = File::create(OUTPUT_PATH_SAMPLES)?;
    write_samples_as_csv(&tasks, &mut sample_output_file, &task_samples)?;

    // Bucket samples
    let bucketed_samples = bucket_samples(&tasks, &task_samples, BUCKET_COUNT);

    // Draw histogram
    write_histogram_as_ascii_art(&tasks, &bucketed_samples);

    // Write histogram data to CSV
    let mut histogram_output_file = File::create(OUTPUT_PATH_HISTOGRAM)?;
    write_histogram_as_csv(&mut histogram_output_file, &tasks, &bucketed_samples);

    // Calculate stats on samples
    let mut stats_output_file = File::create(OUTPUT_PATH_STATS)?;
    let stats = write_stats_as_csv(&mut stats_output_file, tasks.iter().zip(task_samples.iter()));
    dbg!(stats);

    Ok(())
}

fn parse_tasks_from_csv(csv_text: &str) -> Result<Vec<Task>, ()> {
    let mut lines = csv_text.lines();

    // Process header line
    let header_line = lines.next().ok_or(())?;
    let headers = header_line.split(",");
    let mut column_idx_name = None;
    let mut column_idx_min = None;
    let mut column_idx_mode = None;
    let mut column_idx_max = None;
    for (column_idx, column) in headers.enumerate() {
        dbg!(column_idx, column);
        match column.to_lowercase().as_str() {
            "name" => column_idx_name = Some(column_idx),
            "min" => column_idx_min = Some(column_idx),
            "mode" => column_idx_mode = Some(column_idx),
            "max" => column_idx_max = Some(column_idx),
            _ => (),
        }
    }
    if column_idx_name == None {
        return Err(());
    }
    if column_idx_min == None {
        return Err(());
    }
    if column_idx_mode == None {
        return Err(());
    }
    if column_idx_max == None {
        return Err(());
    }

    let mut result = Vec::new();
    // Process tasks
    for task_line in lines {
        let cells: Vec<&str> = task_line.split(",").collect();
        result.push(Task {
            name: cells[column_idx_name.unwrap()].to_owned(),
            min: cells[column_idx_min.unwrap()].parse().or(Err(()))?,
            mode: cells[column_idx_mode.unwrap()].parse().or(Err(()))?,
            max: cells[column_idx_max.unwrap()].parse().or(Err(()))?,
        });
    }

    Ok(result)
}

// CONSIDER: Why operate on an iterator here? Consider operating on a single (Task, TaskSamples)
// pair.
fn write_stats_as_csv<'a>(
    sink: &mut impl Write,
    task_samples: impl Iterator<Item = (&'a Task, &'a TaskSamples)>,
) {
    for (task, samples) in task_samples {
        // CONSIDER: moving this sort up the call stack, or even performing it incrementally as
        // samples are produced.
        let mut sorted_samples = samples.samples.clone();
        sorted_samples.sort_by(f64::total_cmp);

        let get_percentile = |percentile: usize| {
            let sample_idx = ((samples.samples.len() - 1) * percentile) / 100;
            sorted_samples[sample_idx]
        };

        // Percentiles
        let mut percentiles = Vec::new();
        for percentile_idx in 0..PERCENTILES_COUNT {
            let percentile = percentile_idx * 100 / (PERCENTILES_COUNT - 1);
            let value = get_percentile(percentile);
            println!(
                "{percentile}, {}, {}, {percentile}, {value}",
                samples.samples.len(),
                task.name
            );
            percentiles.push((percentile, value));
        }

        // IQR
        let iqr_lower = get_percentile(25);
        let iqr_upper = get_percentile(75);
        let iqr_range = iqr_upper - iqr_lower;
        println!(
            "IQR: 25%: {}, 75%: {}, range: {}",
            iqr_lower,
            iqr_upper,
            iqr_upper - iqr_lower
        );

        // MEDIAN
        let median = get_percentile(50);
        println!("Median: {}", get_percentile(50));

        // MEAN
        let mean = samples.samples.iter().sum::<f64>() / samples.samples.len() as f64;
        println!("Mean: {mean}");

        // Stdev
        let mut stdev = 0.0;
        for sample in &samples.samples {
            stdev += (sample - mean).powi(2) / samples.samples.len() as f64;
        }
        println!("Stdev: {stdev}");

        write!(sink, "{}\n", task.name);
        for (percentile, value) in percentiles {
            write!(sink, "percentile: {percentile},{value}\n");
        }
        write!(sink, "iqr_lower, {iqr_lower}\n");
        write!(sink, "iqr_upper, {iqr_upper}\n");
        write!(sink, "iqr_range, {iqr_range}\n");
        write!(sink, "mean, {mean}\n");
        write!(sink, "stdev, {stdev}\n");
        write!(sink, "\n");
    }
}

fn write_samples_as_csv<Sink: Write>(
    tasks: &Vec<Task>,
    sink: &mut Sink,
    samples: &Vec<TaskSamples>,
) -> std::io::Result<()> {
    write!(sink, "Sample")?;
    for column_header in tasks.iter().map(|t| t.name.as_str()) {
        write!(sink, "{column_header},")?;
    }
    write!(sink, "\n")?;
    for sample_idx in 0..SAMPLE_COUNT {
        write!(sink, "{sample_idx},")?;
        for (_, task_samples) in samples.iter().enumerate() {
            write!(sink, "{},", task_samples.samples[sample_idx])?;
        }
        write!(sink, "\n")?;
    }
    Ok(())
}

fn write_histogram_as_ascii_art(tasks: &[Task], bucketed_samples: &Vec<BucketedSamples>) {
    for (task_idx, task) in tasks.iter().enumerate() {
        println!("{}", task.name);
        let bucketed_samples = &bucketed_samples[task_idx];
        let largest_bucket_sample_count = bucketed_samples
            .buckets
            .iter()
            .map(|b| b.len())
            .max()
            .unwrap();
        for (bucket_idx, bucket) in bucketed_samples.buckets.iter().enumerate() {
            let bucket_start = task.min + (bucketed_samples.bucket_size * bucket_idx as f64);
            let bucket_end = bucket_start + bucketed_samples.bucket_size;
            let bucket_sample_count = bucket.len();
            let bucket_line_size =
                (CHART_LINE_LENGTH * bucket_sample_count) / largest_bucket_sample_count;
            print!("{bucket_start:6.2}-{bucket_end:5.2}: ");
            for _ in 0..bucket_line_size {
                print!("#");
            }
            println!();
            // println!("{bucket_idx},bucket {bucket_start:.2}-{bucket_end:.2},{bucket_sample_count}");
        }
    }
}

fn write_histogram_as_csv(
    sink: &mut impl Write,
    tasks: &[Task],
    bucketed_samples: &Vec<BucketedSamples>,
) {
    for (task_idx, task) in tasks.iter().enumerate() {
        write!(sink, "{}\n", task.name);
        let bucketed_samples = &bucketed_samples[task_idx];
        for (bucket_idx, bucket) in bucketed_samples.buckets.iter().enumerate() {
            let bucket_start = task.min + (bucketed_samples.bucket_size * bucket_idx as f64);
            let bucket_end = bucket_start + bucketed_samples.bucket_size;
            let bucket_sample_count = bucket.len();
            write!(
                sink,
                "{bucket_start:.2}-{bucket_end:.2}, {bucket_sample_count}\n"
            );
        }
        write!(sink, "\n");
    }
}

fn bucket_samples(
    tasks: &[Task],
    samples: &Vec<TaskSamples>,
    bucket_count: usize,
) -> Vec<BucketedSamples> {
    let mut task_bucketed_samples = Vec::new();
    for (task_idx, task) in tasks.iter().enumerate() {
        // HACK
        let (bucket_size, min, max) = if task.min == 0.0 && task.max == 0.0 {
            // let min = samples[task_idx].samples.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let min = 0.0;
            let max = samples[task_idx]
                .samples
                .iter()
                .max_by(|a, b| a.total_cmp(b))
                .unwrap();
            let bucket_size = (max - min) / bucket_count as f64;
            (bucket_size, min, *max)
        } else {
            (
                (task.max - task.min) / bucket_count as f64,
                task.min,
                task.max,
            )
        };
        let mut buckets = vec![Vec::new(); bucket_count];
        for sample in &samples[task_idx].samples {
            let bucket_idx = usize::min(((sample - min) / bucket_size) as usize, BUCKET_COUNT - 1);
            // HACK
            buckets[bucket_idx].push(*sample);
        }
        task_bucketed_samples.push(BucketedSamples {
            buckets,
            bucket_size,
        });
    }
    task_bucketed_samples
}

fn sample_task_estimates(tasks: &[Task], sample_count: usize) -> Vec<TaskSamples> {
    let mut rng = thread_rng();
    let mut task_samples = Vec::new();
    for task in tasks {
        let mut samples = Vec::new();
        for _sample_idx in 0..sample_count {
            let task_value = triangular_distribution_inv_cdf(
                rng.gen_range(0.0..=1.0),
                task.min,
                task.mode,
                task.max,
            );
            samples.push(task_value);
        }
        task_samples.push(TaskSamples { samples });
    }
    let mut samples = Vec::new();
    for sample_idx in 0..sample_count {
        let mut sum = 0.0;
        for task_sample in &task_samples {
            sum += task_sample.samples[sample_idx];
        }
        samples.push(sum);
    }
    task_samples.push(TaskSamples { samples });
    task_samples
}

fn triangular_distribution_inv_cdf(probability: f64, min: f64, mode: f64, max: f64) -> f64 {
    assert!(probability >= 0.0 && probability <= 1.0);
    let cdf_at_mode = (mode - min) / (max - min);
    if probability <= cdf_at_mode {
        min + f64::sqrt(probability * (mode - min) * (max - min))
    } else {
        max - f64::sqrt((max - min) * (max - mode) * (1.0 - probability))
    }
}

fn plot() {
    const SAMPLES: i32 = 1_000_000;
    const MIN: f64 = 0f64;
    const MODE: f64 = 20f64;
    const MAX: f64 = 50f64;

    let mut buckets = vec![0; 100];

    for i in 0..=SAMPLES {
        let t = 1.0 / SAMPLES as f64 * i as f64;
        let x = triangular_distribution_inv_cdf(t, MIN, MODE, MAX);
        buckets[x as usize] += 1;
        // println!("{t} => {x}");
    }
    for bucket in buckets {
        println!("{bucket}");
    }
}

#[derive(Debug)]
struct Task {
    name: String,
    min: f64,
    mode: f64,
    max: f64,
}

struct TaskSamples {
    samples: Vec<f64>,
}

struct BucketedSamples {
    buckets: Vec<Vec<f64>>,
    bucket_size: f64,
}
