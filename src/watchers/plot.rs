use csv::Reader;
use std::error::Error;

#[derive(Debug, Clone)]
struct Row {
    iteration: usize,
    absolute: Option<f64>,
    relative: Option<f64>,
    value: Option<f64>,
    tag: String,
}

fn load_csv(path: &str) -> Result<Vec<Row>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;

    let mut rows = Vec::new();

    for result in rdr.records() {
        let r = result?;

        let row = Row {
            iteration: r.get(0).unwrap().parse()?,
            absolute: parse_opt(r.get(1)),
            relative: parse_opt(r.get(2)),
            value: parse_opt(r.get(3)),
            tag: r.get(4).unwrap_or("").to_string(),
        };

        rows.push(row);
    }

    Ok(rows)
}

fn parse_opt(s: Option<&str>) -> Option<f64> {
    match s {
        Some(v) if !v.is_empty() => v.parse().ok(),
        _ => None,
    }
}
