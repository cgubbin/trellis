use crate::watchers::csv_file::{load_csv, Row};

use plotters::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

#[derive(Copy, Clone)]
pub enum PlotKind {
    Absolute,
    Relative,
    Metric,
}

fn value_for<F: Copy>(kind: PlotKind, row: &Row<F>) -> Option<F> {
    match kind {
        PlotKind::Absolute => row.absolute,
        PlotKind::Relative => row.relative,
        PlotKind::Metric => row.metric,
    }
}

fn title_for(kind: PlotKind) -> &'static str {
    match kind {
        PlotKind::Absolute => "Absolute Error",
        PlotKind::Relative => "Relative Error",
        PlotKind::Metric => "Metric",
    }
}

fn output_name(kind: PlotKind) -> &'static str {
    match kind {
        PlotKind::Absolute => "absolute_error.png",
        PlotKind::Relative => "relative_error.png",
        PlotKind::Metric => "metric.png",
    }
}

pub fn plot_csv(
    csv_file: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let rows: Vec<Row<f64>> = load_csv(csv_file)?;
    let output_dir = output_dir.as_ref();

    for kind in [PlotKind::Absolute, PlotKind::Relative, PlotKind::Metric] {
        plot_kind(&rows, output_dir, kind)?;
    }

    Ok(())
}

fn plot_kind(rows: &[Row<f64>], output_dir: &Path, kind: PlotKind) -> Result<(), Box<dyn Error>> {
    let mut series: BTreeMap<String, Vec<(usize, f64)>> = BTreeMap::new();

    for row in rows {
        if let Some(v) = value_for(kind, row) {
            series
                .entry(row.kind.clone())
                .or_default()
                .push((row.iteration, v));
        }
    }

    if series.is_empty() {
        return Ok(());
    }

    let all_values: Vec<f64> = series
        .values()
        .flat_map(|s| s.iter().map(|(_, y)| *y))
        .collect();

    if all_values.is_empty() {
        return Ok(());
    }

    let x_max = series
        .values()
        .flat_map(|s| s.iter().map(|(x, _)| *x))
        .max()
        .unwrap_or(1);

    let path = output_dir.join(output_name(kind));
    let root = BitMapBackend::new(&path, (1200, 800)).into_drawing_area();
    root.fill(&WHITE)?;

    match kind {
        PlotKind::Metric => {
            let y_min = all_values.iter().copied().fold(f64::INFINITY, f64::min);
            let y_max = all_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);

            let mut chart = ChartBuilder::on(&root)
                .margin(15)
                .caption(title_for(kind), ("sans-serif", 35))
                .x_label_area_size(40)
                .y_label_area_size(70)
                .build_cartesian_2d(0usize..x_max.max(1), y_min..y_max)?;

            chart
                .configure_mesh()
                .x_desc("Iteration")
                .y_desc(title_for(kind))
                .draw()?;

            for (idx, (tag, values)) in series.iter().enumerate() {
                let color = Palette99::pick(idx);

                chart
                    .draw_series(LineSeries::new(values.iter().copied(), &color))?
                    .label(tag.clone())
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
            }

            chart.configure_series_labels().border_style(BLACK).draw()?;
        }

        PlotKind::Absolute | PlotKind::Relative => {
            let positive_values: Vec<f64> = all_values.into_iter().filter(|v| *v > 0.0).collect();

            if positive_values.is_empty() {
                return Ok(());
            }

            let y_min = positive_values
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min)
                .log10();

            let y_max = positive_values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
                .log10();

            let mut chart = ChartBuilder::on(&root)
                .margin(15)
                .caption(title_for(kind), ("sans-serif", 35))
                .x_label_area_size(40)
                .y_label_area_size(70)
                .build_cartesian_2d(0usize..x_max.max(1), y_min..y_max)?;

            chart
                .configure_mesh()
                .x_desc("Iteration")
                .y_desc(format!("log10({})", title_for(kind)))
                .draw()?;

            for (idx, (tag, values)) in series.iter().enumerate() {
                let color = Palette99::pick(idx);

                let transformed = values
                    .iter()
                    .filter(|(_, y)| *y > 0.0)
                    .map(|(x, y)| (*x, y.log10()));

                chart
                    .draw_series(LineSeries::new(transformed, &color))?
                    .label(tag.clone())
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
            }

            chart.configure_series_labels().border_style(BLACK).draw()?;
        }
    }

    root.present()?;
    Ok(())
}
