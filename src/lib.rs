use rand::{thread_rng, Rng};
use std::{fmt, io::Write};
use wasm_bindgen::prelude::*;

const BUCKET_COUNT: usize = 35;
const CHART_LINE_LENGTH: usize = 50;
const PERCENTILES_COUNT: usize = 11;
const SEPARATORS: [char; 3] = [',', '\t', ';'];

#[wasm_bindgen]
pub fn process_input(input: &str, sample_count: usize) -> String {
    let mut result_buffer = Vec::<u8>::new();
    // Parse CSV
    let tasks = parse_tasks_from_csv(input).unwrap();

    // Generate samples
    let total_task = Task {
        name: "Total".to_string(),
        min: 0.0,
        mode: 0.0,
        max: 0.0,
    };
    let sampled_tasks = run_monte_carlo(&tasks, sample_count, &total_task);

    // // Output samples to file
    // let mut sample_output_file = File::create(OUTPUT_PATH_SAMPLES)?;
    // write_samples_as_csv(&mut sample_output_file, &sampled_tasks)?;

    // Bucket samples
    let bucketed_samples = bucket_samples(&sampled_tasks[&sampled_tasks.len() - 1..], BUCKET_COUNT);

    // Draw histogram
    let msg = "Histogram of total values:";
    println!("{msg}\n{}", "=".repeat(msg.len()));
    write_histogram_as_ascii_art(
        &mut result_buffer,
        bucketed_samples.last().unwrap(),
        CHART_LINE_LENGTH,
    )
    .unwrap();

    // // Write histogram data to CSV
    // let mut histogram_output_file = File::create(OUTPUT_PATH_HISTOGRAM)?;
    // write_histogram_as_csv(
    //     &mut histogram_output_file,
    //     &bucketed_samples.last().unwrap(),
    // ).unwrap();

    // Calculate stats on samples
    let stats = calculate_stats(sampled_tasks.last().unwrap(), PERCENTILES_COUNT).unwrap();
    // let mut stats_output_file = File::create(OUTPUT_PATH_STATS)?;
    // stats.write_as_csv(&mut stats_output_file)?;
    stats.write_as_ascii(&mut result_buffer).unwrap();
    String::from_utf8(result_buffer).unwrap()
}

pub fn parse_tasks_from_csv(csv_text: &str) -> Result<Vec<Task>, Error> {
    struct HeaderInfo {
        column_idx_name: usize,
        column_idx_min: usize,
        column_idx_mode: usize,
        column_idx_max: usize,
    }

    fn parse_header_with_separator(header_line: &str, separator: char) -> Result<HeaderInfo, ()> {
        let headers = header_line.split(separator);
        let mut column_idx_name = None;
        let mut column_idx_min = None;
        let mut column_idx_mode = None;
        let mut column_idx_max = None;
        for (column_idx, column) in headers.enumerate() {
            match column.to_lowercase().as_str() {
                "name" => column_idx_name = Some(column_idx),
                "min" => column_idx_min = Some(column_idx),
                "mode" => column_idx_mode = Some(column_idx),
                "max" => column_idx_max = Some(column_idx),
                _ => (),
            }
        }
        if column_idx_name.is_none()
            || column_idx_min.is_none()
            || column_idx_mode.is_none()
            || column_idx_max.is_none()
        {
            return Err(());
        }
        Ok(HeaderInfo {
            column_idx_name: column_idx_name.unwrap(),
            column_idx_min: column_idx_min.unwrap(),
            column_idx_mode: column_idx_mode.unwrap(),
            column_idx_max: column_idx_max.unwrap(),
        })
    }

    let mut lines = csv_text.lines();

    // Process header line
    let mut header_info = None;
    let mut separator = SEPARATORS[0];
    let header_line = lines.next().ok_or(Error {
        msg: "no header line found".to_string(),
    })?;
    for separator_to_try in SEPARATORS {
        if let Ok(r) = parse_header_with_separator(header_line, separator_to_try) {
            header_info = Some(r);
            separator = separator_to_try;
            break;
        }
    }
    let header_info = header_info.ok_or(Error {
        msg: "invalid header".to_string(),
    })?;

    let mut result = Vec::new();
    // Process tasks
    for task_line in lines {
        let cells: Vec<&str> = task_line.split(separator).collect();
        // FIXME: Unwraps
        result.push(Task {
            name: cells[header_info.column_idx_name].to_owned(),
            min: cells[header_info.column_idx_min].parse().unwrap(),
            mode: cells[header_info.column_idx_mode].parse().unwrap(),
            max: cells[header_info.column_idx_max].parse().unwrap(),
        });
    }

    Ok(result)
}

pub fn calculate_stats(
    sampled_task: &SampledTask,
    percentile_count: usize,
) -> Result<Stats, std::io::Error> {
    // CONSIDER: moving this sort up the call stack, or even performing it incrementally as
    // samples are produced.
    let mut sorted_samples = sampled_task.samples.clone();
    sorted_samples.sort_by(f64::total_cmp);

    let get_percentile = |percentile: usize| {
        let sample_idx = ((sampled_task.samples.len() - 1) * percentile) / 100;
        sorted_samples[sample_idx]
    };

    // Percentiles
    let mut percentiles = Vec::new();
    for percentile_idx in 0..percentile_count {
        let percentile = percentile_idx * 100 / (percentile_count - 1);
        let value = get_percentile(percentile);
        percentiles.push((percentile, value));
    }

    // IQR
    let iqr_lower = get_percentile(25);
    let iqr_upper = get_percentile(75);
    let iqr_range = iqr_upper - iqr_lower;

    // MEDIAN
    let median = get_percentile(50);

    // MEAN
    let mean = sampled_task.sum / sampled_task.samples.len() as f64;

    // Stdev
    let mut stdev = 0.0;
    for sample in &sampled_task.samples {
        stdev += (sample - mean).powi(2) / sampled_task.samples.len() as f64;
    }

    Ok(Stats {
        iqr_lower,
        iqr_upper,
        iqr_range,
        mean,
        median,
        stdev,
        percentiles,
    })
}

pub fn write_samples_as_csv(
    sink: &mut impl Write,
    sampled_tasks: &Vec<SampledTask>,
) -> std::io::Result<()> {
    write!(sink, "Sample")?;
    for column_header in sampled_tasks.iter().map(|t| t.task.name.as_str()) {
        write!(sink, "{column_header},")?;
    }
    writeln!(sink)?;
    for sample_idx in 0..sampled_tasks[0].samples.len() {
        write!(sink, "{sample_idx},")?;
        for sampled_task in sampled_tasks {
            write!(sink, "{},", sampled_task.samples[sample_idx])?;
        }
        writeln!(sink)?;
    }
    Ok(())
}

pub fn write_histogram_as_ascii_art(
    sink: &mut impl Write,
    bucketed_sampled_task: &BucketedSamples,
    chart_line_length: usize,
) -> std::io::Result<()> {
    let bucket_half_range = bucketed_sampled_task.bucket_size / 2.0;
    writeln!(
        sink,
        "Each line represents bucket midpoint ± {bucket_half_range:.3}"
    )?;
    let sampled_task = bucketed_sampled_task.sampled_task;
    let largest_bucket_sample_count = bucketed_sampled_task
        .buckets
        .iter()
        .map(|b| b.len())
        .max()
        .unwrap();
    for (bucket_idx, bucket) in bucketed_sampled_task.buckets.iter().enumerate() {
        let bucket_start =
            sampled_task.min + (bucketed_sampled_task.bucket_size * bucket_idx as f64);
        let bucket_midpoint = bucket_start + bucket_half_range;
        let bucket_sample_count = bucket.len();
        let bucket_line_size =
            (chart_line_length * bucket_sample_count) / largest_bucket_sample_count;
        write!(sink, "{bucket_midpoint:6.2}: ")?;
        for _ in 0..bucket_line_size {
            write!(sink, "#")?;
        }
        writeln!(sink)?;
    }
    Ok(())
}

pub fn write_histogram_as_csv(
    sink: &mut impl Write,
    bucketed_sampled_task: &BucketedSamples,
) -> Result<(), std::io::Error> {
    let sampled_task = bucketed_sampled_task.sampled_task;
    writeln!(sink, "{}", sampled_task.task.name)?;
    for (bucket_idx, bucket) in bucketed_sampled_task.buckets.iter().enumerate() {
        let bucket_start =
            sampled_task.task.min + (bucketed_sampled_task.bucket_size * bucket_idx as f64);
        let bucket_end = bucket_start + bucketed_sampled_task.bucket_size;
        let bucket_sample_count = bucket.len();
        writeln!(
            sink,
            "{bucket_start:.2}-{bucket_end:.2}, {bucket_sample_count}"
        )?;
    }
    writeln!(sink)?;
    Ok(())
}

pub fn bucket_samples<'a>(
    sampled_tasks: &'a [SampledTask],
    bucket_count: usize,
) -> Vec<BucketedSamples<'a>> {
    let mut task_bucketed_samples = Vec::new();
    for sampled_task in sampled_tasks {
        let bucket_size = (sampled_task.max - sampled_task.min) / bucket_count as f64;
        let mut buckets = vec![Vec::new(); bucket_count];
        for sample in &sampled_task.samples {
            let bucket_idx = usize::min(
                ((sample - sampled_task.min) / bucket_size) as usize,
                bucket_count - 1,
            );
            // HACK
            buckets[bucket_idx].push(*sample);
        }
        task_bucketed_samples.push(BucketedSamples {
            sampled_task,
            buckets,
            bucket_size,
        });
    }
    task_bucketed_samples
}

pub fn run_monte_carlo<'a>(
    tasks: &'a [Task],
    sample_count: usize,
    total_task: &'a Task,
) -> Vec<SampledTask<'a>> {
    let mut rng = thread_rng();
    let mut sampled_tasks = Vec::new();

    // Sample each task
    for task in tasks {
        let mut samples = Vec::new();
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut sum = 0.0;
        for _sample_idx in 0..sample_count {
            let sample_value = triangular_distribution_inv_cdf(
                rng.gen_range(0.0..=1.0),
                task.min,
                task.mode,
                task.max,
            );
            min = min.min(sample_value);
            max = max.max(sample_value);
            sum += sample_value;
            samples.push(sample_value);
        }
        sampled_tasks.push(SampledTask {
            samples,
            min,
            max,
            sum,
            task,
        })
    }

    // Sum task samples to produce totals
    let mut task_sums = Vec::new();
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut total_sum = 0.0;
    for sample_idx in 0..sample_count {
        let mut tasks_sum = 0.0;
        for task_sample in &sampled_tasks {
            tasks_sum += task_sample.samples[sample_idx];
        }
        min = min.min(tasks_sum);
        max = max.max(tasks_sum);
        total_sum += tasks_sum;
        task_sums.push(tasks_sum);
    }
    sampled_tasks.push(SampledTask {
        samples: task_sums,
        min,
        max,
        sum: total_sum,
        task: total_task,
    });
    sampled_tasks
}

fn triangular_distribution_inv_cdf(probability: f64, min: f64, mode: f64, max: f64) -> f64 {
    assert!((0.0..=1.0).contains(&probability));
    let cdf_at_mode = (mode - min) / (max - min);
    if probability <= cdf_at_mode {
        min + f64::sqrt(probability * (mode - min) * (max - min))
    } else {
        max - f64::sqrt((max - min) * (max - mode) * (1.0 - probability))
    }
}

#[derive(Debug)]
pub struct Task {
    pub name: String,
    pub min: f64,
    pub mode: f64,
    pub max: f64,
}

pub struct BucketedSamples<'a> {
    sampled_task: &'a SampledTask<'a>,
    buckets: Vec<Vec<f64>>,
    bucket_size: f64,
}

pub struct SampledTask<'a> {
    samples: Vec<f64>,
    min: f64,
    max: f64,
    sum: f64,
    task: &'a Task,
}

pub struct Stats {
    iqr_lower: f64,
    iqr_upper: f64,
    iqr_range: f64,
    mean: f64,
    median: f64,
    stdev: f64,
    percentiles: Vec<(usize, f64)>,
}
impl Stats {
    pub fn write_as_csv(&self, sink: &mut impl Write) -> Result<(), std::io::Error> {
        for (percentile, value) in &self.percentiles {
            writeln!(sink, "percentile: {percentile},{value}")?;
        }
        writeln!(sink, "iqr_lower, {}", self.iqr_lower)?;
        writeln!(sink, "iqr_upper, {}", self.iqr_upper)?;
        writeln!(sink, "iqr_range, {}", self.iqr_range)?;
        writeln!(sink, "mean, {}", self.mean)?;
        writeln!(sink, "median, {}", self.median)?;
        writeln!(sink, "stdev, {}", self.stdev)?;
        writeln!(sink)?;
        Ok(())
    }

    pub fn write_as_ascii(&self, sink: &mut impl Write) -> Result<(), std::io::Error> {
        // Percentiles
        let msg = "Percentiles";
        write!(sink, "\n{msg}\n{}\n", "=".repeat(msg.len()))?;
        for (percentile, value) in &self.percentiles {
            writeln!(sink, "| {percentile:3} | {value:5.2} |")?;
        }

        // Statistics
        let msg = "Statistics";
        write!(sink, "\n{msg}\n{}\n", "=".repeat(msg.len()))?;
        let stats = vec![
            ("Lower quartile", format!("{:.2}", self.iqr_lower)),
            ("Upper quartile", format!("{:.2}", self.iqr_upper)),
            ("Interquartile range", format!("{:.2}", self.iqr_range)),
            ("Median", format!("{:.2}", self.median)),
            ("Mean", format!("{:.2}", self.mean)),
            ("Standard deviation", format!("{:.2}", self.stdev)),
        ];
        let longest_key = stats.iter().map(|(key, _)| key.len()).max().unwrap();
        let longest_value = &stats.iter().map(|(_, value)| value.len()).max().unwrap();
        for (key, value) in stats {
            writeln!(
                sink,
                "| {key:0$} | {value:1$} |",
                longest_key, longest_value
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}
