use core::f64;
use std::path::PathBuf;

use trellis_runner::{prelude::*, ErrorEstimate};

struct DummyCalculation {}

struct DummyProblem {}

struct DummyState {
    param: Option<Vec<f64>>,
    cost: f64,
    is_initialised: bool,
    iter: usize,
}

impl UserState for DummyState {
    type Float = f64;
    type Param = Vec<f64>;
    fn new() -> Self {
        Self {
            param: None,
            cost: f64::MAX,
            is_initialised: true,
            iter: 0,
        }
    }

    fn is_initialised(&self) -> bool {
        self.is_initialised
    }

    fn get_param(&self) -> Option<&Self::Param> {
        self.param.as_ref()
    }

    fn update(&mut self) -> ErrorEstimate<Self::Float> {
        self.iter += 1;
        ErrorEstimate(self.cost)
    }

    fn last_was_best(&mut self) {}
}

#[derive(Debug)]
enum DummyError {
    TypeA,
}

impl std::fmt::Display for DummyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DummyError {}

impl Calculation<DummyProblem, DummyState> for DummyCalculation {
    type Error = DummyError;
    type Output = bool;
    const NAME: &'static str = "My dumb calculation";
    fn initialise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        state: DummyState,
    ) -> Result<DummyState, Self::Error> {
        println!("initialising");
        Ok(state)
    }

    fn next(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        mut state: DummyState,
    ) -> Result<DummyState, Self::Error> {
        std::thread::sleep(std::time::Duration::from_millis(100));

        state.cost = (-((state.iter as f64) / 100.0)).exp();

        Ok(state)
    }

    fn finalise(
        &mut self,
        _problem: &mut Problem<DummyProblem>,
        _state: DummyState,
    ) -> Result<Self::Output, Self::Error> {
        Ok(true)
    }
}

#[tokio::test]
#[cfg(feature = "tokio")]
async fn problems_run_successfully() {
    let calculation = DummyCalculation {};
    let problem = DummyProblem {};

    // let iden = "calculation_time".to_string();
    // let outdir = PathBuf::from(r"/Users/cgubbin/sensorium/tooling/runner/out/");
    //
    // let config = PlotConfig {
    //     x_limits: 0.0..100.0,
    //     y_limits: None,
    //     x_label: "Iteration".into(),
    //     y_label: "Measure".into(),
    //     title: "Optimisation Progress".into(),
    // };
    //
    let cancellation_token = tokio_util::sync::CancellationToken::new();

    let runner = calculation
        .build_for(problem)
        .with_cancellation_token(cancellation_token.clone())
        .configure(|state| state.max_iters(100))
        // .attach_observer(
        //     FileWriter::new(
        //         outdir.clone(),
        //         iden.clone(),
        //         WriteToFileSerializer::JSON,
        //         Target::Measure,
        //     ),
        //     Frequency::Always,
        // )
        // .attach_observer(
        //     PlotGenerator::measure(outdir, iden, config),
        //     Frequency::Always,
        // )
        .finalise()
        .expect("failed to build problem");

    // tokio::task::spawn_blocking(move || runner.run());
    let result = runner.run();
    dbg!(result);

    std::thread::sleep(std::time::Duration::from_millis(10000));
    cancellation_token.cancel();
}
