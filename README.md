# trellis-runner

## Trellis

Trellis is a generic execution engine for iterative numerical algorithms.

Rather than implementing optimisation loops, integration loops or search loops
directly, a procedure describes a single iteration of an algorithm while
Trellis manages execution, convergence, termination, observation and
checkpointing.

The library separates the concerns of:

- **algorithm implementation** (`Procedure`)
- **problem definition** (the problem supplied to the procedure)
- **algorithm state** (`UserState`)
- **execution control** (policies)
- **instrumentation** (observers)

This separation allows algorithms to focus solely on their numerical method,
while Trellis provides a reusable execution framework.

### Execution model

Every Trellis calculation follows the same execution model:

```
                Problem
                   │
                   ▼
             ┌───────────┐
             │ Procedure │
             └─────┬─────┘
                   │
             updates state
                   │
                   ▼
             ┌───────────┐
             │ UserState │
             └─────┬─────┘
                   │
           emits Progress events
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
  Engine Policies        Observers
        │
        ▼
   Engine Actions
```

The principal abstractions are:

- **Problem** — the specific problem instance being solved.
- **Procedure** — performs a single iteration of the algorithm.
- **UserState** — stores the evolving state of the computation and reports
  progress.
- **Policies** — inspect progress and control execution.
- **Observers** — inspect progress without affecting execution.

### A simple example

A calculation is configured using the builder returned by
[`GenerateBuilder::build_for`]:

```rust
#
#
#
#
#
#
#
#
#
let engine = Solver
    .build_for(Problem)
    .with_initial_state(State::default())
    .and_policy(RelativeTolerancePolicy::new(1e-8, 10))
    .and_policy(MaxIterationPolicy::new(10_000))
    .finalise();

let result = engine.run();
```

The builder configures the execution environment rather than the numerical
algorithm itself.

### Procedures

A [`Procedure`] implements the numerical algorithm.

Each call to `step()` performs a single iteration of the algorithm, while
`finalise()` converts the final algorithm state into the value returned to the
caller.

Both infallible and fallible procedures are supported.

### User state

[`UserState`] stores the evolving state of the computation.

In addition to algorithm-specific data, it reports progress to the engine via
[`Progress`], allowing policies and observers to monitor execution.

States implementing [`Snapshotable`] can additionally participate in
checkpointing, allowing long-running computations to be resumed.

### Policies

Policies control solver execution.

During a run, the engine collects progress emitted by the procedure and
passes it to one or more policies. Policies inspect this information and
decide whether the solver should:

- continue running,
- terminate successfully,
- terminate early,
- request a checkpoint,
- or perform another engine action.

Policies influence execution.

Observers do not.

```
Progress ──► Policy ──► Engine Action
           │
           └────► Observer
```

Multiple policies may be attached simultaneously.

The engine stops as soon as any policy requests termination.

Custom policies can be created implementing the [`EnginePolicy`] trait.

#### Built-in policies

| Policy | Description |
|---------|-------------|
| `MaxIterationPolicy` | Stops after a fixed number of iterations. |
| `TimeoutPolicy` | Stops after a maximum wall-clock duration. |
| `AbsoluteTolerancePolicy` | Stops when the mean absolute error over a rolling window falls below a tolerance. |
| `RelativeTolerancePolicy` | Stops when the mean relative error over a rolling window falls below a tolerance. |
| `TargetValuePolicy` | Stops when the mean distance to a target value remains below a tolerance. |
| `NoProgressPolicy` | Stops when no meaningful improvement has been observed for a specified number of iterations. |
| `StagnationPolicy` | Stops when improvement over a rolling window falls below a relative threshold. |
| `CheckpointPolicy` | Requests periodic checkpoint generation. |

### Observers

Observers receive every event emitted by the engine but never influence
execution.

Typical applications include:

- structured logging,
- tracing,
- CSV export,
- plotting,
- metrics collection,
- progress reporting,
- custom visualisation.

### Checkpointing

User states implementing [`Snapshotable`] may be checkpointed during
execution.

Checkpoints may be requested by policies or generated manually, allowing
interrupted computations to be resumed.

### Extending Trellis

Trellis is designed to be extended through traits.

Most applications only need to implement:

- [`Procedure`] to define the numerical algorithm,
- [`UserState`] to store algorithm state,
- [`EnginePolicy`] for custom stopping criteria,
- [`Observe`] for custom instrumentation.

These components compose naturally, allowing new algorithms, policies and
observers to be combined without modifying the execution engine itself.

License: MIT
