use trellis_runner::{
    CancellationGuard, GenerateBuilder, MaxIterationPolicy, Procedure, Progress,
    ProgressDiagnostics, StagnationPolicy, UserState,
};

#[derive(Clone)]
pub struct LinearRegressionProblem {
    pub data: Vec<(f64, f64)>,
}

#[derive(Clone, Debug)]
pub struct LSState {
    a: f64,
    b: f64,
    loss: f64,
}

impl Default for LSState {
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            loss: f64::INFINITY,
        }
    }
}

impl UserState for LSState {
    type Float = f64;

    fn progress(&self) -> Progress<Self::Float> {
        Progress::Report {
            measure: self.loss,
            diagnostics: ProgressDiagnostics {
                gradient_norm: Some((self.a.powi(2) + self.b.powi(2)).sqrt()),
                step_size: Some(0.01),
                ..Default::default()
            },
        }
    }
}

/// Simple linear regression via gradient descent
pub struct LeastSquares;

impl Procedure<LinearRegressionProblem> for LeastSquares {
    type State = LSState;
    type Output = (f64, f64);

    const NAME: &'static str = "Least Squares Optimisation";

    fn step(
        &self,
        problem: &mut LinearRegressionProblem,
        state: &mut Self::State,
        _guard: CancellationGuard<'_>,
    ) {
        let lr = 0.01;
        let mut da = 0.0;
        let mut db = 0.0;
        let mut loss = 0.0;

        for (x, y) in &problem.data {
            let pred = state.a * *x + state.b;
            let err = pred - *y;

            loss += err * err;
            da += err * *x;
            db += err;
        }

        state.a -= lr * da;
        state.b -= lr * db;
        state.loss = loss;
    }

    fn finalise(&self, _: &mut LinearRegressionProblem, state: &Self::State) -> Self::Output {
        (state.a, state.b)
    }
}

fn main() {
    let problem = LinearRegressionProblem {
        data: vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)],
    };

    let result = LeastSquares
        .build_for(problem)
        .with_initial_state(LSState::default())
        .and_policy(MaxIterationPolicy::new(3000))
        .and_policy(StagnationPolicy::new(10))
        .finalise()
        .run();

    println!("fit: {:?}", result);
}
