### `sdk/src/activity_context.rs`
```rs
use crate::app_data::AppData;

use prost_types::{Duration, Timestamp};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration as StdDuration, SystemTime},
};
use temporal_client::Priority;
use temporal_sdk_core_api::Worker;
use temporal_sdk_core_protos::{
    coresdk::{ActivityHeartbeat, activity_task},
    temporal::api::common::v1::{Payload, RetryPolicy, WorkflowExecution},
    utilities::TryIntoOrNone,
};
use tokio_util::sync::CancellationToken;

/// Used within activities to get info, heartbeat management etc.
#[derive(Clone)]
pub struct ActContext {
    worker: Arc<dyn Worker>,
    app_data: Arc<AppData>,
    cancellation_token: CancellationToken,
    input: Vec<Payload>,
    heartbeat_details: Vec<Payload>,
    header_fields: HashMap<String, Payload>,
    info: ActivityInfo,
}

#[derive(Clone)]
pub struct ActivityInfo {
    pub task_token: Vec<u8>,
    pub workflow_type: String,
    pub workflow_namespace: String,
    pub workflow_execution: Option<WorkflowExecution>,
    pub activity_id: String,
    pub activity_type: String,
    pub task_queue: String,
    pub heartbeat_timeout: Option<StdDuration>,
    /// Time activity was scheduled by a workflow
    pub scheduled_time: Option<SystemTime>,
    /// Time of activity start
    pub started_time: Option<SystemTime>,
    /// Time of activity timeout
    pub deadline: Option<SystemTime>,
    /// Attempt starts from 1, and increase by 1 for every retry, if retry policy is specified.
    pub attempt: u32,
    /// Time this attempt at the activity was scheduled
    pub current_attempt_scheduled_time: Option<SystemTime>,
    pub retry_policy: Option<RetryPolicy>,
    pub is_local: bool,
    /// Priority of this activity. If unset uses [Priority::default]
    pub priority: Priority,
}

impl ActContext {
    /// Construct new Activity Context, returning the context and the first argument to the activity
    /// (which may be a default [Payload]).
    pub(crate) fn new(
        worker: Arc<dyn Worker>,
        app_data: Arc<AppData>,
        cancellation_token: CancellationToken,
        task_queue: String,
        task_token: Vec<u8>,
        task: activity_task::Start,
    ) -> (Self, Payload) {
        let activity_task::Start {
            workflow_namespace,
            workflow_type,
            workflow_execution,
            activity_id,
            activity_type,
            header_fields,
            mut input,
            heartbeat_details,
            scheduled_time,
            current_attempt_scheduled_time,
            started_time,
            attempt,
            schedule_to_close_timeout,
            start_to_close_timeout,
            heartbeat_timeout,
            retry_policy,
            is_local,
            priority,
        } = task;
        let deadline = calculate_deadline(
            scheduled_time.as_ref(),
            started_time.as_ref(),
            start_to_close_timeout.as_ref(),
            schedule_to_close_timeout.as_ref(),
        );
        let first_arg = input.pop().unwrap_or_default();

        (
            ActContext {
                worker,
                app_data,
                cancellation_token,
                input,
                heartbeat_details,
                header_fields,
                info: ActivityInfo {
                    task_token,
                    task_queue,
                    workflow_type,
                    workflow_namespace,
                    workflow_execution,
                    activity_id,
                    activity_type,
                    heartbeat_timeout: heartbeat_timeout.try_into_or_none(),
                    scheduled_time: scheduled_time.try_into_or_none(),
                    started_time: started_time.try_into_or_none(),
                    deadline,
                    attempt,
                    current_attempt_scheduled_time: current_attempt_scheduled_time
                        .try_into_or_none(),
                    retry_policy,
                    is_local,
                    priority: priority.map(Into::into).unwrap_or_default(),
                },
            },
            first_arg,
        )
    }

    /// Returns a future the completes if and when the activity this was called inside has been
    /// cancelled
    pub async fn cancelled(&self) {
        self.cancellation_token.clone().cancelled().await
    }

    /// Returns true if this activity has already been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }

    /// Retrieve extra parameters to the Activity. The first input is always popped and passed to
    /// the Activity function for the currently executing activity. However, if more parameters are
    /// passed, perhaps from another language's SDK, explicit access is available from extra_inputs
    pub fn extra_inputs(&mut self) -> &mut [Payload] {
        &mut self.input
    }

    /// Extract heartbeat details from last failed attempt. This is used in combination with retry policy.
    pub fn get_heartbeat_details(&self) -> &[Payload] {
        &self.heartbeat_details
    }

    /// RecordHeartbeat sends heartbeat for the currently executing activity
    pub fn record_heartbeat(&self, details: Vec<Payload>) {
        self.worker.record_activity_heartbeat(ActivityHeartbeat {
            task_token: self.info.task_token.clone(),
            details,
        })
    }

    /// Get activity info of the executing activity
    pub fn get_info(&self) -> &ActivityInfo {
        &self.info
    }

    /// Get headers attached to this activity
    pub fn headers(&self) -> &HashMap<String, Payload> {
        &self.header_fields
    }

    /// Get custom Application Data
    pub fn app_data<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.app_data.get::<T>()
    }
}

/// Deadline calculation.  This is a port of
/// https://github.com/temporalio/sdk-go/blob/8651550973088f27f678118f997839fb1bb9e62f/internal/activity.go#L225
fn calculate_deadline(
    scheduled_time: Option<&Timestamp>,
    started_time: Option<&Timestamp>,
    start_to_close_timeout: Option<&Duration>,
    schedule_to_close_timeout: Option<&Duration>,
) -> Option<SystemTime> {
    match (
        scheduled_time,
        started_time,
        start_to_close_timeout,
        schedule_to_close_timeout,
    ) {
        (
            Some(scheduled),
            Some(started),
            Some(start_to_close_timeout),
            Some(schedule_to_close_timeout),
        ) => {
            let scheduled: SystemTime = maybe_convert_timestamp(scheduled)?;
            let started: SystemTime = maybe_convert_timestamp(started)?;
            let start_to_close_timeout: StdDuration = (*start_to_close_timeout).try_into().ok()?;
            let schedule_to_close_timeout: StdDuration =
                (*schedule_to_close_timeout).try_into().ok()?;

            let start_to_close_deadline: SystemTime =
                started.checked_add(start_to_close_timeout)?;
            if schedule_to_close_timeout > StdDuration::ZERO {
                let schedule_to_close_deadline =
                    scheduled.checked_add(schedule_to_close_timeout)?;
                // Minimum of the two deadlines.
                if schedule_to_close_deadline < start_to_close_deadline {
                    Some(schedule_to_close_deadline)
                } else {
                    Some(start_to_close_deadline)
                }
            } else {
                Some(start_to_close_deadline)
            }
        }
        _ => None,
    }
}

/// Helper function lifted from prost_types::Timestamp implementation to prevent double cloning in
/// error construction
fn maybe_convert_timestamp(timestamp: &Timestamp) -> Option<SystemTime> {
    let mut timestamp = *timestamp;
    timestamp.normalize();

    let system_time = if timestamp.seconds >= 0 {
        std::time::UNIX_EPOCH.checked_add(StdDuration::from_secs(timestamp.seconds as u64))
    } else {
        std::time::UNIX_EPOCH.checked_sub(StdDuration::from_secs((-timestamp.seconds) as u64))
    };

    system_time.and_then(|system_time| {
        system_time.checked_add(StdDuration::from_nanos(timestamp.nanos as u64))
    })
}
```
### `sdk/src/app_data.rs`
```rs
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
};

/// A Wrapper Type for workflow and activity app data
#[derive(Default)]
pub(crate) struct AppData {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl AppData {
    /// Insert an item, overwritting duplicates
    pub(crate) fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(downcast_owned)
    }

    /// Get a reference to a type in the map
    pub(crate) fn get<T: 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref())
    }
}

impl fmt::Debug for AppData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppData").finish()
    }
}

fn downcast_owned<T: Send + Sync + 'static>(boxed: Box<dyn Any + Send + Sync>) -> Option<T> {
    boxed.downcast().ok().map(|boxed| *boxed)
}
```
### `sdk/src/interceptors.rs`
```rs
//! User-definable interceptors are defined in this module

use crate::Worker;
use anyhow::bail;
use std::sync::{Arc, OnceLock};
use temporal_sdk_core_protos::{
    coresdk::{
        workflow_activation::{WorkflowActivation, remove_from_cache::EvictionReason},
        workflow_completion::WorkflowActivationCompletion,
    },
    temporal::api::common::v1::Payload,
};

/// Implementors can intercept certain actions that happen within the Worker.
///
/// Advanced usage only.
#[async_trait::async_trait(?Send)]
pub trait WorkerInterceptor {
    /// Called every time a workflow activation completes (just before sending the completion to
    /// core).
    async fn on_workflow_activation_completion(&self, _completion: &WorkflowActivationCompletion) {}
    /// Called after the worker has initiated shutdown and the workflow/activity polling loops
    /// have exited, but just before waiting for the inner core worker shutdown
    fn on_shutdown(&self, _sdk_worker: &Worker) {}
    /// Called every time a workflow is about to be activated
    async fn on_workflow_activation(
        &self,
        _activation: &WorkflowActivation,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

/// Supports the composition of interceptors
pub struct InterceptorWithNext {
    inner: Box<dyn WorkerInterceptor>,
    next: Option<Box<InterceptorWithNext>>,
}

impl InterceptorWithNext {
    /// Create from an existing interceptor, can be used to initialize a chain of interceptors
    pub fn new(inner: Box<dyn WorkerInterceptor>) -> Self {
        Self { inner, next: None }
    }

    /// Sets the next interceptor, and then returns that interceptor, wrapped by
    /// [InterceptorWithNext]. You can keep calling this method on it to extend the chain.
    pub fn set_next(&mut self, next: Box<dyn WorkerInterceptor>) -> &mut InterceptorWithNext {
        self.next.insert(Box::new(Self::new(next)))
    }
}

#[async_trait::async_trait(?Send)]
impl WorkerInterceptor for InterceptorWithNext {
    async fn on_workflow_activation_completion(&self, c: &WorkflowActivationCompletion) {
        self.inner.on_workflow_activation_completion(c).await;
        if let Some(next) = &self.next {
            next.on_workflow_activation_completion(c).await;
        }
    }

    fn on_shutdown(&self, w: &Worker) {
        self.inner.on_shutdown(w);
        if let Some(next) = &self.next {
            next.on_shutdown(w);
        }
    }

    async fn on_workflow_activation(&self, a: &WorkflowActivation) -> Result<(), anyhow::Error> {
        self.inner.on_workflow_activation(a).await?;
        if let Some(next) = &self.next {
            next.on_workflow_activation(a).await?;
        }
        Ok(())
    }
}

/// An interceptor which causes the worker's run function to exit early if nondeterminism errors are
/// encountered
pub struct FailOnNondeterminismInterceptor {}
#[async_trait::async_trait(?Send)]
impl WorkerInterceptor for FailOnNondeterminismInterceptor {
    async fn on_workflow_activation(
        &self,
        activation: &WorkflowActivation,
    ) -> Result<(), anyhow::Error> {
        if matches!(
            activation.eviction_reason(),
            Some(EvictionReason::Nondeterminism)
        ) {
            bail!(
                "Workflow is being evicted because of nondeterminism! {}",
                activation
            );
        }
        Ok(())
    }
}

/// An interceptor that allows you to fetch the exit value of the workflow if and when it is set
#[derive(Default)]
pub struct ReturnWorkflowExitValueInterceptor {
    result_value: Arc<OnceLock<Payload>>,
}

impl ReturnWorkflowExitValueInterceptor {
    /// Can be used to fetch the workflow result if/when it is determined
    pub fn get_result_handle(&self) -> Arc<OnceLock<Payload>> {
        self.result_value.clone()
    }
}

#[async_trait::async_trait(?Send)]
impl WorkerInterceptor for ReturnWorkflowExitValueInterceptor {
    async fn on_workflow_activation_completion(&self, c: &WorkflowActivationCompletion) {
        if let Some(v) = c.complete_workflow_execution_value() {
            let _ = self.result_value.set(v.clone());
        }
    }
}
```
### `sdk/src/lib.rs`
```rs
#![warn(missing_docs)] // error if there are missing docs

//! This crate defines an alpha-stage Temporal Rust SDK.
//!
//! Currently defining activities and running an activity-only worker is the most stable code.
//! Workflow definitions exist and running a workflow worker works, but the API is still very
//! unstable.
//!
//! An example of running an activity worker:
//! ```no_run
//! use std::{str::FromStr, sync::Arc};
//! use temporal_sdk::{sdk_client_options, ActContext, Worker};
//! use temporal_sdk_core::{init_worker, Url, CoreRuntime};
//! use temporal_sdk_core_api::{
//!     worker::{WorkerConfigBuilder, WorkerVersioningStrategy},
//!     telemetry::TelemetryOptionsBuilder
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server_options = sdk_client_options(Url::from_str("http://localhost:7233")?).build()?;
//!
//!     let client = server_options.connect("default", None).await?;
//!
//!     let telemetry_options = TelemetryOptionsBuilder::default().build()?;
//!     let runtime = CoreRuntime::new_assume_tokio(telemetry_options)?;
//!
//!     let worker_config = WorkerConfigBuilder::default()
//!         .namespace("default")
//!         .task_queue("task_queue")
//!         .versioning_strategy(WorkerVersioningStrategy::None { build_id: "rust-sdk".to_owned() })
//!         .build()?;
//!
//!     let core_worker = init_worker(&runtime, worker_config, client)?;
//!
//!     let mut worker = Worker::new_from_core(Arc::new(core_worker), "task_queue");
//!     worker.register_activity(
//!         "echo_activity",
//!         |_ctx: ActContext, echo_me: String| async move { Ok(echo_me) },
//!     );
//!
//!     worker.run().await?;
//!
//!     Ok(())
//! }
//! ```

#[macro_use]
extern crate tracing;

mod activity_context;
mod app_data;
pub mod interceptors;
mod workflow_context;
mod workflow_future;

pub use activity_context::ActContext;
pub use temporal_client::Namespace;
use tracing::{Instrument, Span, field};
pub use workflow_context::{
    ActivityOptions, CancellableFuture, ChildWorkflow, ChildWorkflowOptions, LocalActivityOptions,
    NexusOperationOptions, PendingChildWorkflow, Signal, SignalData, SignalWorkflowOptions,
    StartedChildWorkflow, TimerOptions, WfContext,
};

use crate::{
    interceptors::WorkerInterceptor,
    workflow_context::{ChildWfCommon, NexusUnblockData, StartedNexusOperation},
};
use anyhow::{Context, anyhow, bail};
use app_data::AppData;
use futures_util::{FutureExt, StreamExt, TryFutureExt, TryStreamExt, future::BoxFuture};
use serde::Serialize;
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display, Formatter},
    future::Future,
    panic::AssertUnwindSafe,
    sync::Arc,
    time::Duration,
};
use temporal_client::ClientOptionsBuilder;
use temporal_sdk_core::Url;
use temporal_sdk_core_api::{Worker as CoreWorker, errors::PollError};
use temporal_sdk_core_protos::{
    TaskToken,
    coresdk::{
        ActivityTaskCompletion, AsJsonPayloadExt, FromJsonPayloadExt,
        activity_result::{ActivityExecutionResult, ActivityResolution},
        activity_task::{ActivityTask, activity_task},
        child_workflow::ChildWorkflowResult,
        common::NamespacedWorkflowExecution,
        nexus::NexusOperationResult,
        workflow_activation::{
            WorkflowActivation,
            resolve_child_workflow_execution_start::Status as ChildWorkflowStartStatus,
            resolve_nexus_operation_start, workflow_activation_job::Variant,
        },
        workflow_commands::{ContinueAsNewWorkflowExecution, WorkflowCommand, workflow_command},
        workflow_completion::WorkflowActivationCompletion,
    },
    temporal::api::{
        common::v1::Payload,
        enums::v1::WorkflowTaskFailedCause,
        failure::v1::{Failure, failure},
    },
};
use tokio::{
    sync::{
        Notify,
        mpsc::{UnboundedSender, unbounded_channel},
        oneshot,
    },
    task::JoinError,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_util::sync::CancellationToken;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns a [ClientOptionsBuilder] with required fields set to appropriate values
/// for the Rust SDK.
pub fn sdk_client_options(url: impl Into<Url>) -> ClientOptionsBuilder {
    let mut builder = ClientOptionsBuilder::default();
    builder
        .target_url(url)
        .client_name("temporal-rust".to_string())
        .client_version(VERSION.to_string());

    builder
}

/// A worker that can poll for and respond to workflow tasks by using [WorkflowFunction]s,
/// and activity tasks by using [ActivityFunction]s
pub struct Worker {
    common: CommonWorker,
    workflow_half: WorkflowHalf,
    activity_half: ActivityHalf,
    app_data: Option<AppData>,
}

struct CommonWorker {
    worker: Arc<dyn CoreWorker>,
    task_queue: String,
    worker_interceptor: Option<Box<dyn WorkerInterceptor>>,
}

struct WorkflowHalf {
    /// Maps run id to cached workflow state
    workflows: RefCell<HashMap<String, WorkflowData>>,
    /// Maps workflow type to the function for executing workflow runs with that ID
    workflow_fns: RefCell<HashMap<String, WorkflowFunction>>,
    workflow_removed_from_map: Notify,
}
struct WorkflowData {
    /// Channel used to send the workflow activations
    activation_chan: UnboundedSender<WorkflowActivation>,
}

struct WorkflowFutureHandle<F: Future<Output = Result<WorkflowResult<Payload>, JoinError>>> {
    join_handle: F,
    run_id: String,
}

struct ActivityHalf {
    /// Maps activity type to the function for executing activities of that type
    activity_fns: HashMap<String, ActivityFunction>,
    task_tokens_to_cancels: HashMap<TaskToken, CancellationToken>,
}

impl Worker {
    /// Create a new Rust SDK worker from a core worker
    pub fn new_from_core(worker: Arc<dyn CoreWorker>, task_queue: impl Into<String>) -> Self {
        Self {
            common: CommonWorker {
                worker,
                task_queue: task_queue.into(),
                worker_interceptor: None,
            },
            workflow_half: WorkflowHalf {
                workflows: Default::default(),
                workflow_fns: Default::default(),
                workflow_removed_from_map: Default::default(),
            },
            activity_half: ActivityHalf {
                activity_fns: Default::default(),
                task_tokens_to_cancels: Default::default(),
            },
            app_data: Some(Default::default()),
        }
    }

    /// Returns the task queue name this worker polls on
    pub fn task_queue(&self) -> &str {
        &self.common.task_queue
    }

    /// Return a handle that can be used to initiate shutdown.
    /// TODO: Doc better after shutdown changes
    pub fn shutdown_handle(&self) -> impl Fn() + use<> {
        let w = self.common.worker.clone();
        move || w.initiate_shutdown()
    }

    /// Register a Workflow function to invoke when the Worker is asked to run a workflow of
    /// `workflow_type`
    pub fn register_wf(
        &mut self,
        workflow_type: impl Into<String>,
        wf_function: impl Into<WorkflowFunction>,
    ) {
        self.workflow_half
            .workflow_fns
            .get_mut()
            .insert(workflow_type.into(), wf_function.into());
    }

    /// Register an Activity function to invoke when the Worker is asked to run an activity of
    /// `activity_type`
    pub fn register_activity<A, R, O>(
        &mut self,
        activity_type: impl Into<String>,
        act_function: impl IntoActivityFunc<A, R, O>,
    ) {
        self.activity_half.activity_fns.insert(
            activity_type.into(),
            ActivityFunction {
                act_func: act_function.into_activity_fn(),
            },
        );
    }

    /// Insert Custom App Context for Workflows and Activities
    pub fn insert_app_data<T: Send + Sync + 'static>(&mut self, data: T) {
        self.app_data.as_mut().map(|a| a.insert(data));
    }

    /// Runs the worker. Eventually resolves after the worker has been explicitly shut down,
    /// or may return early with an error in the event of some unresolvable problem.
    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let shutdown_token = CancellationToken::new();
        let (common, wf_half, act_half, app_data) = self.split_apart();
        let safe_app_data = Arc::new(
            app_data
                .take()
                .ok_or_else(|| anyhow!("app_data should exist on run"))?,
        );
        let (wf_future_tx, wf_future_rx) = unbounded_channel();
        let (completions_tx, completions_rx) = unbounded_channel();
        let wf_future_joiner = async {
            UnboundedReceiverStream::new(wf_future_rx)
                .map(Result::<_, anyhow::Error>::Ok)
                .try_for_each_concurrent(
                    None,
                    |WorkflowFutureHandle {
                         join_handle,
                         run_id,
                     }| {
                        let wf_half = &*wf_half;
                        async move {
                            join_handle.await??;
                            debug!(run_id=%run_id, "Removing workflow from cache");
                            wf_half.workflows.borrow_mut().remove(&run_id);
                            wf_half.workflow_removed_from_map.notify_one();
                            Ok(())
                        }
                    },
                )
                .await
                .context("Workflow futures encountered an error")
        };
        let wf_completion_processor = async {
            UnboundedReceiverStream::new(completions_rx)
                .map(Ok)
                .try_for_each_concurrent(None, |completion| async {
                    if let Some(ref i) = common.worker_interceptor {
                        i.on_workflow_activation_completion(&completion).await;
                    }
                    common.worker.complete_workflow_activation(completion).await
                })
                .map_err(anyhow::Error::from)
                .await
                .context("Workflow completions processor encountered an error")
        };
        tokio::try_join!(
            // Workflow polling loop
            async {
                loop {
                    let activation = match common.worker.poll_workflow_activation().await {
                        Err(PollError::ShutDown) => {
                            break;
                        }
                        o => o?,
                    };
                    if let Some(ref i) = common.worker_interceptor {
                        i.on_workflow_activation(&activation).await?;
                    }
                    if let Some(wf_fut) = wf_half
                        .workflow_activation_handler(
                            common,
                            shutdown_token.clone(),
                            activation,
                            &completions_tx,
                        )
                        .await?
                        && wf_future_tx.send(wf_fut).is_err()
                    {
                        panic!("Receive half of completion processor channel cannot be dropped");
                    }
                }
                // Tell still-alive workflows to evict themselves
                shutdown_token.cancel();
                // It's important to drop these so the future and completion processors will
                // terminate.
                drop(wf_future_tx);
                drop(completions_tx);
                Result::<_, anyhow::Error>::Ok(())
            },
            // Only poll on the activity queue if activity functions have been registered. This
            // makes tests which use mocks dramatically more manageable.
            async {
                if !act_half.activity_fns.is_empty() {
                    loop {
                        let activity = common.worker.poll_activity_task().await;
                        if matches!(activity, Err(PollError::ShutDown)) {
                            break;
                        }
                        act_half.activity_task_handler(
                            common.worker.clone(),
                            safe_app_data.clone(),
                            common.task_queue.clone(),
                            activity?,
                        )?;
                    }
                };
                Result::<_, anyhow::Error>::Ok(())
            },
            wf_future_joiner,
            wf_completion_processor,
        )?;

        info!("Polling loops exited");
        if let Some(i) = self.common.worker_interceptor.as_ref() {
            i.on_shutdown(self);
        }
        self.common.worker.shutdown().await;
        debug!("Worker shutdown complete");
        self.app_data = Some(
            Arc::try_unwrap(safe_app_data)
                .map_err(|_| anyhow!("some references of AppData exist on worker shutdown"))?,
        );
        Ok(())
    }

    /// Set a [WorkerInterceptor]
    pub fn set_worker_interceptor(&mut self, interceptor: impl WorkerInterceptor + 'static) {
        self.common.worker_interceptor = Some(Box::new(interceptor));
    }

    /// Turns this rust worker into a new worker with all the same workflows and activities
    /// registered, but with a new underlying core worker. Can be used to swap the worker for
    /// a replay worker, change task queues, etc.
    pub fn with_new_core_worker(&mut self, new_core_worker: Arc<dyn CoreWorker>) {
        self.common.worker = new_core_worker;
    }

    /// Returns number of currently cached workflows as understood by the SDK. Importantly, this
    /// is not the same as understood by core, though they *should* always be in sync.
    pub fn cached_workflows(&self) -> usize {
        self.workflow_half.workflows.borrow().len()
    }

    fn split_apart(
        &mut self,
    ) -> (
        &mut CommonWorker,
        &mut WorkflowHalf,
        &mut ActivityHalf,
        &mut Option<AppData>,
    ) {
        (
            &mut self.common,
            &mut self.workflow_half,
            &mut self.activity_half,
            &mut self.app_data,
        )
    }
}

impl WorkflowHalf {
    #[allow(clippy::type_complexity)]
    async fn workflow_activation_handler(
        &self,
        common: &CommonWorker,
        shutdown_token: CancellationToken,
        mut activation: WorkflowActivation,
        completions_tx: &UnboundedSender<WorkflowActivationCompletion>,
    ) -> Result<
        Option<
            WorkflowFutureHandle<
                impl Future<Output = Result<WorkflowResult<Payload>, JoinError>> + use<>,
            >,
        >,
        anyhow::Error,
    > {
        let mut res = None;
        let run_id = activation.run_id.clone();

        // If the activation is to init a workflow, create a new workflow driver for it,
        // using the function associated with that workflow id
        if let Some(sw) = activation.jobs.iter_mut().find_map(|j| match j.variant {
            Some(Variant::InitializeWorkflow(ref mut sw)) => Some(sw),
            _ => None,
        }) {
            let workflow_type = &sw.workflow_type;
            let (wff, activations) = {
                let wf_fns_borrow = self.workflow_fns.borrow();

                let Some(wf_function) = wf_fns_borrow.get(workflow_type) else {
                    warn!("Workflow type {workflow_type} not found");

                    completions_tx
                        .send(WorkflowActivationCompletion::fail(
                            run_id,
                            format!("Workflow type {workflow_type} not found").into(),
                            Some(WorkflowTaskFailedCause::WorkflowWorkerUnhandledFailure),
                        ))
                        .expect("Completion channel intact");
                    return Ok(None);
                };

                wf_function.start_workflow(
                    common.worker.get_config().namespace.clone(),
                    common.task_queue.clone(),
                    std::mem::take(sw),
                    completions_tx.clone(),
                )
            };
            let jh = tokio::spawn(async move {
                tokio::select! {
                    r = wff.fuse() => r,
                    // TODO: This probably shouldn't abort early, as it could cause an in-progress
                    //  complete to abort. Send synthetic remove activation
                    _ = shutdown_token.cancelled() => {
                        Ok(WfExitValue::Evicted)
                    }
                }
            });
            res = Some(WorkflowFutureHandle {
                join_handle: jh,
                run_id: run_id.clone(),
            });
            loop {
                // It's possible that we've got a new initialize workflow action before the last
                // future for this run finished evicting, as a result of how futures might be
                // interleaved. In that case, just wait until it's not in the map, which should be
                // a matter of only a few `poll` calls.
                if self.workflows.borrow_mut().contains_key(&run_id) {
                    self.workflow_removed_from_map.notified().await;
                } else {
                    break;
                }
            }
            self.workflows.borrow_mut().insert(
                run_id.clone(),
                WorkflowData {
                    activation_chan: activations,
                },
            );
        }

        // The activation is expected to apply to some workflow we know about. Use it to
        // unblock things and advance the workflow.
        if let Some(dat) = self.workflows.borrow_mut().get_mut(&run_id) {
            dat.activation_chan
                .send(activation)
                .expect("Workflow should exist if we're sending it an activation");
        } else {
            // When we failed to start a workflow, we never inserted it into the cache. But core
            // sends us a `RemoveFromCache` job when we mark the StartWorkflow workflow activation
            // as a failure, which we need to complete. Other SDKs add the workflow to the cache
            // even when the workflow type is unknown/not found. To circumvent this, we simply mark
            // any RemoveFromCache job for workflows that are not in the cache as complete.
            if activation.jobs.len() == 1
                && matches!(
                    activation.jobs.first().map(|j| &j.variant),
                    Some(Some(Variant::RemoveFromCache(_)))
                )
            {
                completions_tx
                    .send(WorkflowActivationCompletion::from_cmds(run_id, vec![]))
                    .expect("Completion channel intact");
                return Ok(None);
            }

            // In all other cases, we want to error as the runtime could be in an inconsistent state
            // at this point.
            bail!(
                "Got activation {:?} for unknown workflow {}",
                activation,
                run_id
            );
        };

        Ok(res)
    }
}

impl ActivityHalf {
    /// Spawns off a task to handle the provided activity task
    fn activity_task_handler(
        &mut self,
        worker: Arc<dyn CoreWorker>,
        app_data: Arc<AppData>,
        task_queue: String,
        activity: ActivityTask,
    ) -> Result<(), anyhow::Error> {
        match activity.variant {
            Some(activity_task::Variant::Start(start)) => {
                let act_fn = self
                    .activity_fns
                    .get(&start.activity_type)
                    .ok_or_else(|| {
                        anyhow!(
                            "No function registered for activity type {}",
                            start.activity_type
                        )
                    })?
                    .clone();
                let span = info_span!(
                    "RunActivity",
                    "otel.name" = format!("RunActivity:{}", start.activity_type),
                    "otel.kind" = "server",
                    "temporalActivityID" = start.activity_id,
                    "temporalWorkflowID" = field::Empty,
                    "temporalRunID" = field::Empty,
                );
                let ct = CancellationToken::new();
                let task_token = activity.task_token;
                self.task_tokens_to_cancels
                    .insert(task_token.clone().into(), ct.clone());

                let (ctx, arg) = ActContext::new(
                    worker.clone(),
                    app_data,
                    ct,
                    task_queue,
                    task_token.clone(),
                    start,
                );

                tokio::spawn(async move {
                    let act_fut = async move {
                        if let Some(info) = &ctx.get_info().workflow_execution {
                            Span::current()
                                .record("temporalWorkflowID", &info.workflow_id)
                                .record("temporalRunID", &info.run_id);
                        }
                        (act_fn.act_func)(ctx, arg).await
                    }
                    .instrument(span);
                    let output = AssertUnwindSafe(act_fut).catch_unwind().await;
                    let result = match output {
                        Err(e) => ActivityExecutionResult::fail(Failure::application_failure(
                            format!("Activity function panicked: {}", panic_formatter(e)),
                            true,
                        )),
                        Ok(Ok(ActExitValue::Normal(p))) => ActivityExecutionResult::ok(p),
                        Ok(Ok(ActExitValue::WillCompleteAsync)) => {
                            ActivityExecutionResult::will_complete_async()
                        }
                        Ok(Err(err)) => match err {
                            ActivityError::Retryable {
                                source,
                                explicit_delay,
                            } => ActivityExecutionResult::fail({
                                let mut f = Failure::application_failure_from_error(source, false);
                                if let Some(d) = explicit_delay
                                    && let Some(failure::FailureInfo::ApplicationFailureInfo(fi)) =
                                        f.failure_info.as_mut()
                                {
                                    fi.next_retry_delay = d.try_into().ok();
                                }
                                f
                            }),
                            ActivityError::Cancelled { details } => {
                                ActivityExecutionResult::cancel_from_details(details)
                            }
                            ActivityError::NonRetryable(nre) => ActivityExecutionResult::fail(
                                Failure::application_failure_from_error(nre, true),
                            ),
                        },
                    };
                    worker
                        .complete_activity_task(ActivityTaskCompletion {
                            task_token,
                            result: Some(result),
                        })
                        .await?;
                    Ok::<_, anyhow::Error>(())
                });
            }
            Some(activity_task::Variant::Cancel(_)) => {
                if let Some(ct) = self
                    .task_tokens_to_cancels
                    .get(activity.task_token.as_slice())
                {
                    ct.cancel();
                }
            }
            None => bail!("Undefined activity task variant"),
        }
        Ok(())
    }
}

#[derive(Debug)]
enum UnblockEvent {
    Timer(u32, TimerResult),
    Activity(u32, Box<ActivityResolution>),
    WorkflowStart(u32, Box<ChildWorkflowStartStatus>),
    WorkflowComplete(u32, Box<ChildWorkflowResult>),
    SignalExternal(u32, Option<Failure>),
    CancelExternal(u32, Option<Failure>),
    NexusOperationStart(u32, Box<resolve_nexus_operation_start::Status>),
    NexusOperationComplete(u32, Box<NexusOperationResult>),
}

/// Result of awaiting on a timer
#[derive(Debug, Copy, Clone)]
pub enum TimerResult {
    /// The timer was cancelled
    Cancelled,
    /// The timer elapsed and fired
    Fired,
}

/// Successful result of sending a signal to an external workflow
#[derive(Debug)]
pub struct SignalExternalOk;
/// Result of awaiting on sending a signal to an external workflow
pub type SignalExternalWfResult = Result<SignalExternalOk, Failure>;

/// Successful result of sending a cancel request to an external workflow
#[derive(Debug)]
pub struct CancelExternalOk;
/// Result of awaiting on sending a cancel request to an external workflow
pub type CancelExternalWfResult = Result<CancelExternalOk, Failure>;

trait Unblockable {
    type OtherDat;

    fn unblock(ue: UnblockEvent, od: Self::OtherDat) -> Self;
}

impl Unblockable for TimerResult {
    type OtherDat = ();
    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::Timer(_, result) => result,
            _ => panic!("Invalid unblock event for timer"),
        }
    }
}

impl Unblockable for ActivityResolution {
    type OtherDat = ();
    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::Activity(_, result) => *result,
            _ => panic!("Invalid unblock event for activity"),
        }
    }
}

impl Unblockable for PendingChildWorkflow {
    // Other data here is workflow id
    type OtherDat = ChildWfCommon;
    fn unblock(ue: UnblockEvent, od: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::WorkflowStart(_, result) => Self {
                status: *result,
                common: od,
            },
            _ => panic!("Invalid unblock event for child workflow start"),
        }
    }
}

impl Unblockable for ChildWorkflowResult {
    type OtherDat = ();
    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::WorkflowComplete(_, result) => *result,
            _ => panic!("Invalid unblock event for child workflow complete"),
        }
    }
}

impl Unblockable for SignalExternalWfResult {
    type OtherDat = ();
    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::SignalExternal(_, maybefail) => {
                maybefail.map_or(Ok(SignalExternalOk), Err)
            }
            _ => panic!("Invalid unblock event for signal external workflow result"),
        }
    }
}

impl Unblockable for CancelExternalWfResult {
    type OtherDat = ();
    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::CancelExternal(_, maybefail) => {
                maybefail.map_or(Ok(CancelExternalOk), Err)
            }
            _ => panic!("Invalid unblock event for signal external workflow result"),
        }
    }
}

type NexusStartResult = Result<StartedNexusOperation, Failure>;
impl Unblockable for NexusStartResult {
    type OtherDat = NexusUnblockData;
    fn unblock(ue: UnblockEvent, od: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::NexusOperationStart(_, result) => match *result {
                resolve_nexus_operation_start::Status::OperationToken(op_token) => {
                    Ok(StartedNexusOperation {
                        operation_token: Some(op_token),
                        unblock_dat: od,
                    })
                }
                resolve_nexus_operation_start::Status::StartedSync(_) => {
                    Ok(StartedNexusOperation {
                        operation_token: None,
                        unblock_dat: od,
                    })
                }
                resolve_nexus_operation_start::Status::CancelledBeforeStart(f) => Err(f),
            },
            _ => panic!("Invalid unblock event for nexus operation"),
        }
    }
}

impl Unblockable for NexusOperationResult {
    type OtherDat = ();

    fn unblock(ue: UnblockEvent, _: Self::OtherDat) -> Self {
        match ue {
            UnblockEvent::NexusOperationComplete(_, result) => *result,
            _ => panic!("Invalid unblock event for nexus operation complete"),
        }
    }
}

/// Identifier for cancellable operations
#[derive(Debug, Clone)]
pub(crate) enum CancellableID {
    Timer(u32),
    Activity(u32),
    LocalActivity(u32),
    ChildWorkflow {
        seqnum: u32,
        reason: String,
    },
    SignalExternalWorkflow(u32),
    ExternalWorkflow {
        seqnum: u32,
        execution: NamespacedWorkflowExecution,
        reason: String,
    },
    /// A nexus operation (waiting for start)
    NexusOp(u32),
}

/// Cancellation IDs that support a reason.
pub(crate) trait SupportsCancelReason {
    /// Returns a new version of this ID with the provided cancellation reason.
    fn with_reason(self, reason: String) -> CancellableID;
}
#[derive(Debug, Clone)]
pub(crate) enum CancellableIDWithReason {
    ChildWorkflow {
        seqnum: u32,
    },
    ExternalWorkflow {
        seqnum: u32,
        execution: NamespacedWorkflowExecution,
    },
}
impl CancellableIDWithReason {
    pub(crate) fn seq_num(&self) -> u32 {
        match self {
            CancellableIDWithReason::ChildWorkflow { seqnum } => *seqnum,
            CancellableIDWithReason::ExternalWorkflow { seqnum, .. } => *seqnum,
        }
    }
}
impl SupportsCancelReason for CancellableIDWithReason {
    fn with_reason(self, reason: String) -> CancellableID {
        match self {
            CancellableIDWithReason::ChildWorkflow { seqnum } => {
                CancellableID::ChildWorkflow { seqnum, reason }
            }
            CancellableIDWithReason::ExternalWorkflow { seqnum, execution } => {
                CancellableID::ExternalWorkflow {
                    seqnum,
                    execution,
                    reason,
                }
            }
        }
    }
}
impl From<CancellableIDWithReason> for CancellableID {
    fn from(v: CancellableIDWithReason) -> Self {
        v.with_reason("".to_string())
    }
}

#[derive(derive_more::From)]
#[allow(clippy::large_enum_variant)]
enum RustWfCmd {
    #[from(ignore)]
    Cancel(CancellableID),
    ForceWFTFailure(anyhow::Error),
    NewCmd(CommandCreateRequest),
    NewNonblockingCmd(workflow_command::Variant),
    SubscribeChildWorkflowCompletion(CommandSubscribeChildWorkflowCompletion),
    SubscribeSignal(String, UnboundedSender<SignalData>),
    RegisterUpdate(String, UpdateFunctions),
    SubscribeNexusOperationCompletion {
        seq: u32,
        unblocker: oneshot::Sender<UnblockEvent>,
    },
}

struct CommandCreateRequest {
    cmd: WorkflowCommand,
    unblocker: oneshot::Sender<UnblockEvent>,
}

struct CommandSubscribeChildWorkflowCompletion {
    seq: u32,
    unblocker: oneshot::Sender<UnblockEvent>,
}

type WfFunc = dyn Fn(WfContext) -> BoxFuture<'static, Result<WfExitValue<Payload>, anyhow::Error>>
    + Send
    + Sync
    + 'static;

/// The user's async function / workflow code
pub struct WorkflowFunction {
    wf_func: Box<WfFunc>,
}

impl<F, Fut, O> From<F> for WorkflowFunction
where
    F: Fn(WfContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<WfExitValue<O>, anyhow::Error>> + Send + 'static,
    O: Serialize,
{
    fn from(wf_func: F) -> Self {
        Self::new(wf_func)
    }
}

impl WorkflowFunction {
    /// Build a workflow function from a closure or function pointer which accepts a [WfContext]
    pub fn new<F, Fut, O>(f: F) -> Self
    where
        F: Fn(WfContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<WfExitValue<O>, anyhow::Error>> + Send + 'static,
        O: Serialize,
    {
        Self {
            wf_func: Box::new(move |ctx: WfContext| {
                (f)(ctx)
                    .map(|r| {
                        r.and_then(|r| {
                            Ok(match r {
                                WfExitValue::ContinueAsNew(b) => WfExitValue::ContinueAsNew(b),
                                WfExitValue::Cancelled => WfExitValue::Cancelled,
                                WfExitValue::Evicted => WfExitValue::Evicted,
                                WfExitValue::Normal(o) => WfExitValue::Normal(o.as_json_payload()?),
                            })
                        })
                    })
                    .boxed()
            }),
        }
    }
}

/// The result of running a workflow
pub type WorkflowResult<T> = Result<WfExitValue<T>, anyhow::Error>;

/// Workflow functions may return these values when exiting
#[derive(Debug, derive_more::From)]
pub enum WfExitValue<T> {
    /// Continue the workflow as a new execution
    #[from(ignore)]
    ContinueAsNew(Box<ContinueAsNewWorkflowExecution>),
    /// Confirm the workflow was cancelled (can be automatic in a more advanced iteration)
    #[from(ignore)]
    Cancelled,
    /// The run was evicted
    #[from(ignore)]
    Evicted,
    /// Finish with a result
    Normal(T),
}

impl<T> WfExitValue<T> {
    /// Construct a [WfExitValue::ContinueAsNew] variant (handles boxing)
    pub fn continue_as_new(can: ContinueAsNewWorkflowExecution) -> Self {
        Self::ContinueAsNew(Box::new(can))
    }
}

/// Activity functions may return these values when exiting
pub enum ActExitValue<T> {
    /// Completion requires an asynchronous callback
    WillCompleteAsync,
    /// Finish with a result
    Normal(T),
}

impl<T: AsJsonPayloadExt> From<T> for ActExitValue<T> {
    fn from(t: T) -> Self {
        Self::Normal(t)
    }
}

type BoxActFn = Arc<
    dyn Fn(ActContext, Payload) -> BoxFuture<'static, Result<ActExitValue<Payload>, ActivityError>>
        + Send
        + Sync,
>;

/// Container for user-defined activity functions
#[derive(Clone)]
pub struct ActivityFunction {
    act_func: BoxActFn,
}

/// Returned as errors from activity functions
#[derive(Debug)]
pub enum ActivityError {
    /// This error can be returned from activities to allow the explicit configuration of certain
    /// error properties. It's also the default error type that arbitrary errors will be converted
    /// into.
    Retryable {
        /// The underlying error
        source: anyhow::Error,
        /// If specified, the next retry (if there is one) will occur after this delay
        explicit_delay: Option<Duration>,
    },
    /// Return this error to indicate your activity is cancelling
    Cancelled {
        /// Some data to save as the cancellation reason
        details: Option<Payload>,
    },
    /// Return this error to indicate that your activity non-retryable
    /// this is a transparent wrapper around anyhow Error so essentially any type of error
    /// could be used here.
    NonRetryable(anyhow::Error),
}

impl<E> From<E> for ActivityError
where
    E: Into<anyhow::Error>,
{
    fn from(source: E) -> Self {
        Self::Retryable {
            source: source.into(),
            explicit_delay: None,
        }
    }
}

impl ActivityError {
    /// Construct a cancelled error without details
    pub fn cancelled() -> Self {
        Self::Cancelled { details: None }
    }
}

/// Closures / functions which can be turned into activity functions implement this trait
pub trait IntoActivityFunc<Args, Res, Out> {
    /// Consume the closure or fn pointer and turned it into a boxed activity function
    fn into_activity_fn(self) -> BoxActFn;
}

impl<A, Rf, R, O, F> IntoActivityFunc<A, Rf, O> for F
where
    F: (Fn(ActContext, A) -> Rf) + Sync + Send + 'static,
    A: FromJsonPayloadExt + Send,
    Rf: Future<Output = Result<R, ActivityError>> + Send + 'static,
    R: Into<ActExitValue<O>>,
    O: AsJsonPayloadExt,
{
    fn into_activity_fn(self) -> BoxActFn {
        let wrapper = move |ctx: ActContext, input: Payload| {
            // Some minor gymnastics are required to avoid needing to clone the function
            match A::from_json_payload(&input) {
                Ok(deser) => self(ctx, deser)
                    .map(|r| {
                        r.and_then(|r| {
                            let exit_val: ActExitValue<O> = r.into();
                            match exit_val {
                                ActExitValue::WillCompleteAsync => {
                                    Ok(ActExitValue::WillCompleteAsync)
                                }
                                ActExitValue::Normal(x) => match x.as_json_payload() {
                                    Ok(v) => Ok(ActExitValue::Normal(v)),
                                    Err(e) => Err(ActivityError::NonRetryable(e)),
                                },
                            }
                        })
                    })
                    .boxed(),
                Err(e) => async move { Err(ActivityError::NonRetryable(e.into())) }.boxed(),
            }
        };
        Arc::new(wrapper)
    }
}

/// Extra information attached to workflow updates
#[derive(Clone)]
pub struct UpdateInfo {
    /// The update's id, unique within the workflow
    pub update_id: String,
    /// Headers attached to the update
    pub headers: HashMap<String, Payload>,
}

/// Context for a workflow update
pub struct UpdateContext {
    /// The workflow context, can be used to do normal workflow things inside the update handler
    pub wf_ctx: WfContext,
    /// Additional update info
    pub info: UpdateInfo,
}

struct UpdateFunctions {
    validator: BoxUpdateValidatorFn,
    handler: BoxUpdateHandlerFn,
}

impl UpdateFunctions {
    pub(crate) fn new<Arg, Res>(
        v: impl IntoUpdateValidatorFunc<Arg> + Sized,
        h: impl IntoUpdateHandlerFunc<Arg, Res> + Sized,
    ) -> Self {
        Self {
            validator: v.into_update_validator_fn(),
            handler: h.into_update_handler_fn(),
        }
    }
}

type BoxUpdateValidatorFn = Box<dyn Fn(&UpdateInfo, &Payload) -> Result<(), anyhow::Error> + Send>;
/// Closures / functions which can be turned into update validation functions implement this trait
pub trait IntoUpdateValidatorFunc<Arg> {
    /// Consume the closure/fn pointer and turn it into an update validator
    fn into_update_validator_fn(self) -> BoxUpdateValidatorFn;
}
impl<A, F> IntoUpdateValidatorFunc<A> for F
where
    A: FromJsonPayloadExt + Send,
    F: (for<'a> Fn(&'a UpdateInfo, A) -> Result<(), anyhow::Error>) + Send + 'static,
{
    fn into_update_validator_fn(self) -> BoxUpdateValidatorFn {
        let wrapper = move |ctx: &UpdateInfo, input: &Payload| match A::from_json_payload(input) {
            Ok(deser) => (self)(ctx, deser),
            Err(e) => Err(e.into()),
        };
        Box::new(wrapper)
    }
}
type BoxUpdateHandlerFn = Box<
    dyn FnMut(UpdateContext, &Payload) -> BoxFuture<'static, Result<Payload, anyhow::Error>> + Send,
>;
/// Closures / functions which can be turned into update handler functions implement this trait
pub trait IntoUpdateHandlerFunc<Arg, Res> {
    /// Consume the closure/fn pointer and turn it into an update handler
    fn into_update_handler_fn(self) -> BoxUpdateHandlerFn;
}
impl<A, F, Rf, R> IntoUpdateHandlerFunc<A, R> for F
where
    A: FromJsonPayloadExt + Send,
    F: (FnMut(UpdateContext, A) -> Rf) + Send + 'static,
    Rf: Future<Output = Result<R, anyhow::Error>> + Send + 'static,
    R: AsJsonPayloadExt,
{
    fn into_update_handler_fn(mut self) -> BoxUpdateHandlerFn {
        let wrapper = move |ctx: UpdateContext, input: &Payload| match A::from_json_payload(input) {
            Ok(deser) => (self)(ctx, deser)
                .map(|r| r.and_then(|r| r.as_json_payload()))
                .boxed(),
            Err(e) => async move { Err(e.into()) }.boxed(),
        };
        Box::new(wrapper)
    }
}

/// Attempts to turn caught panics into something printable
fn panic_formatter(panic: Box<dyn Any>) -> Box<dyn Display> {
    _panic_formatter::<&str>(panic)
}
fn _panic_formatter<T: 'static + PrintablePanicType>(panic: Box<dyn Any>) -> Box<dyn Display> {
    match panic.downcast::<T>() {
        Ok(d) => d,
        Err(orig) => {
            if TypeId::of::<<T as PrintablePanicType>::NextType>()
                == TypeId::of::<EndPrintingAttempts>()
            {
                return Box::new("Couldn't turn panic into a string");
            }
            _panic_formatter::<T::NextType>(orig)
        }
    }
}
trait PrintablePanicType: Display {
    type NextType: PrintablePanicType;
}
impl PrintablePanicType for &str {
    type NextType = String;
}
impl PrintablePanicType for String {
    type NextType = EndPrintingAttempts;
}
struct EndPrintingAttempts {}
impl Display for EndPrintingAttempts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Will never be printed")
    }
}
impl PrintablePanicType for EndPrintingAttempts {
    type NextType = EndPrintingAttempts;
}
```
### `sdk/src/workflow_context/options.rs`
```rs
use std::{collections::HashMap, time::Duration};

use temporal_client::{Priority, WorkflowOptions};
use temporal_sdk_core_protos::{
    coresdk::{
        child_workflow::ChildWorkflowCancellationType,
        nexus::NexusOperationCancellationType,
        workflow_commands::{
            ActivityCancellationType, ScheduleActivity, ScheduleLocalActivity,
            ScheduleNexusOperation, StartChildWorkflowExecution, WorkflowCommand,
        },
    },
    temporal::api::{
        common::v1::{Payload, RetryPolicy},
        enums::v1::ParentClosePolicy,
        sdk::v1::UserMetadata,
    },
};
// TODO: Before release, probably best to avoid using proto types entirely here. They're awkward.

pub(crate) trait IntoWorkflowCommand {
    /// Produces a workflow command from some options
    fn into_command(self, seq: u32) -> WorkflowCommand;
}

/// Options for scheduling an activity
#[derive(Default, Debug)]
pub struct ActivityOptions {
    /// Identifier to use for tracking the activity in Workflow history.
    /// The `activityId` can be accessed by the activity function.
    /// Does not need to be unique.
    ///
    /// If `None` use the context's sequence number
    pub activity_id: Option<String>,
    /// Type of activity to schedule
    pub activity_type: String,
    /// Input to the activity
    pub input: Payload,
    /// Task queue to schedule the activity in
    ///
    /// If `None`, use the same task queue as the parent workflow.
    pub task_queue: Option<String>,
    /// Time that the Activity Task can stay in the Task Queue before it is picked up by a Worker.
    /// Do not specify this timeout unless using host specific Task Queues for Activity Tasks are
    /// being used for routing.
    /// `schedule_to_start_timeout` is always non-retryable.
    /// Retrying after this timeout doesn't make sense as it would just put the Activity Task back
    /// into the same Task Queue.
    pub schedule_to_start_timeout: Option<Duration>,
    /// Maximum time of a single Activity execution attempt.
    /// Note that the Temporal Server doesn't detect Worker process failures directly.
    /// It relies on this timeout to detect that an Activity that didn't complete on time.
    /// So this timeout should be as short as the longest possible execution of the Activity body.
    /// Potentially long running Activities must specify `heartbeat_timeout` and heartbeat from the
    /// activity periodically for timely failure detection.
    /// Either this option or `schedule_to_close_timeout` is required.
    pub start_to_close_timeout: Option<Duration>,
    /// Total time that a workflow is willing to wait for Activity to complete.
    /// `schedule_to_close_timeout` limits the total time of an Activity's execution including
    /// retries (use `start_to_close_timeout` to limit the time of a single attempt).
    /// Either this option or `start_to_close_timeout` is required.
    pub schedule_to_close_timeout: Option<Duration>,
    /// Heartbeat interval. Activity must heartbeat before this interval passes after a last
    /// heartbeat or activity start.
    pub heartbeat_timeout: Option<Duration>,
    /// Determines what the SDK does when the Activity is cancelled.
    pub cancellation_type: ActivityCancellationType,
    /// Activity retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Summary of the activity
    pub summary: Option<String>,
    /// Priority for the activity
    pub priority: Option<Priority>,
    /// If true, disable eager execution for this activity
    pub do_not_eagerly_execute: bool,
}

impl IntoWorkflowCommand for ActivityOptions {
    fn into_command(self, seq: u32) -> WorkflowCommand {
        WorkflowCommand {
            variant: Some(
                ScheduleActivity {
                    seq,
                    activity_id: match self.activity_id {
                        None => seq.to_string(),
                        Some(aid) => aid,
                    },
                    activity_type: self.activity_type,
                    task_queue: self.task_queue.unwrap_or_default(),
                    schedule_to_close_timeout: self
                        .schedule_to_close_timeout
                        .and_then(|d| d.try_into().ok()),
                    schedule_to_start_timeout: self
                        .schedule_to_start_timeout
                        .and_then(|d| d.try_into().ok()),
                    start_to_close_timeout: self
                        .start_to_close_timeout
                        .and_then(|d| d.try_into().ok()),
                    heartbeat_timeout: self.heartbeat_timeout.and_then(|d| d.try_into().ok()),
                    cancellation_type: self.cancellation_type as i32,
                    arguments: vec![self.input],
                    retry_policy: self.retry_policy,
                    priority: self.priority.map(Into::into),
                    do_not_eagerly_execute: self.do_not_eagerly_execute,
                    ..Default::default()
                }
                .into(),
            ),
            user_metadata: self.summary.map(|s| UserMetadata {
                summary: Some(s.into()),
                details: None,
            }),
        }
    }
}

/// Options for scheduling a local activity
#[derive(Default, Debug, Clone)]
pub struct LocalActivityOptions {
    /// Identifier to use for tracking the activity in Workflow history.
    /// The `activityId` can be accessed by the activity function.
    /// Does not need to be unique.
    ///
    /// If `None` use the context's sequence number
    pub activity_id: Option<String>,
    /// Type of activity to schedule
    pub activity_type: String,
    /// Input to the activity
    // TODO: Make optional
    pub input: Payload,
    /// Retry policy
    pub retry_policy: RetryPolicy,
    /// Override attempt number rather than using 1.
    /// Ideally we would not expose this in a released Rust SDK, but it's needed for test.
    pub attempt: Option<u32>,
    /// Override schedule time when doing timer backoff.
    /// Ideally we would not expose this in a released Rust SDK, but it's needed for test.
    pub original_schedule_time: Option<prost_types::Timestamp>,
    /// Retry backoffs over this amount will use a timer rather than a local retry
    pub timer_backoff_threshold: Option<Duration>,
    /// How the activity will cancel
    pub cancel_type: ActivityCancellationType,
    /// Indicates how long the caller is willing to wait for local activity completion. Limits how
    /// long retries will be attempted. When not specified defaults to the workflow execution
    /// timeout (which may be unset).
    pub schedule_to_close_timeout: Option<Duration>,
    /// Limits time the local activity can idle internally before being executed. That can happen if
    /// the worker is currently at max concurrent local activity executions. This timeout is always
    /// non retryable as all a retry would achieve is to put it back into the same queue. Defaults
    /// to `schedule_to_close_timeout` if not specified and that is set. Must be <=
    /// `schedule_to_close_timeout` when set, if not, it will be clamped down.
    pub schedule_to_start_timeout: Option<Duration>,
    /// Maximum time the local activity is allowed to execute after the task is dispatched. This
    /// timeout is always retryable. Either or both of `schedule_to_close_timeout` and this must be
    /// specified. If set, this must be <= `schedule_to_close_timeout`, if not, it will be clamped
    /// down.
    pub start_to_close_timeout: Option<Duration>,
}

impl IntoWorkflowCommand for LocalActivityOptions {
    fn into_command(mut self, seq: u32) -> WorkflowCommand {
        // Allow tests to avoid extra verbosity when they don't care about timeouts
        // TODO: Builderize LA options
        self.schedule_to_close_timeout
            .get_or_insert(Duration::from_secs(100));

        WorkflowCommand {
            variant: Some(
                ScheduleLocalActivity {
                    seq,
                    attempt: self.attempt.unwrap_or(1),
                    original_schedule_time: self.original_schedule_time,
                    activity_id: match self.activity_id {
                        None => seq.to_string(),
                        Some(aid) => aid,
                    },
                    activity_type: self.activity_type,
                    arguments: vec![self.input],
                    retry_policy: Some(self.retry_policy),
                    local_retry_threshold: self
                        .timer_backoff_threshold
                        .and_then(|d| d.try_into().ok()),
                    cancellation_type: self.cancel_type.into(),
                    schedule_to_close_timeout: self
                        .schedule_to_close_timeout
                        .and_then(|d| d.try_into().ok()),
                    schedule_to_start_timeout: self
                        .schedule_to_start_timeout
                        .and_then(|d| d.try_into().ok()),
                    start_to_close_timeout: self
                        .start_to_close_timeout
                        .and_then(|d| d.try_into().ok()),
                    ..Default::default()
                }
                .into(),
            ),
            user_metadata: None,
        }
    }
}

/// Options for scheduling a child workflow
#[derive(Default, Debug, Clone)]
pub struct ChildWorkflowOptions {
    /// Workflow ID
    pub workflow_id: String,
    /// Type of workflow to schedule
    pub workflow_type: String,
    /// Task queue to schedule the workflow in
    ///
    /// If `None`, use the same task queue as the parent workflow.
    pub task_queue: Option<String>,
    /// Input to send the child Workflow
    pub input: Vec<Payload>,
    /// Cancellation strategy for the child workflow
    pub cancel_type: ChildWorkflowCancellationType,
    /// Common options
    pub options: WorkflowOptions,
    /// How to respond to parent workflow ending
    pub parent_close_policy: ParentClosePolicy,
    /// Static summary of the child workflow
    pub static_summary: Option<String>,
    /// Static details of the child workflow
    pub static_details: Option<String>,
}

impl IntoWorkflowCommand for ChildWorkflowOptions {
    fn into_command(self, seq: u32) -> WorkflowCommand {
        let user_metadata = if self.static_summary.is_some() || self.static_details.is_some() {
            Some(UserMetadata {
                summary: self.static_summary.map(Into::into),
                details: self.static_details.map(Into::into),
            })
        } else {
            None
        };
        WorkflowCommand {
            variant: Some(
                StartChildWorkflowExecution {
                    seq,
                    workflow_id: self.workflow_id,
                    workflow_type: self.workflow_type,
                    task_queue: self.task_queue.unwrap_or_default(),
                    input: self.input,
                    cancellation_type: self.cancel_type as i32,
                    workflow_id_reuse_policy: self.options.id_reuse_policy as i32,
                    workflow_execution_timeout: self
                        .options
                        .execution_timeout
                        .and_then(|d| d.try_into().ok()),
                    workflow_run_timeout: self
                        .options
                        .execution_timeout
                        .and_then(|d| d.try_into().ok()),
                    workflow_task_timeout: self
                        .options
                        .task_timeout
                        .and_then(|d| d.try_into().ok()),
                    search_attributes: self.options.search_attributes.unwrap_or_default(),
                    cron_schedule: self.options.cron_schedule.unwrap_or_default(),
                    parent_close_policy: self.parent_close_policy as i32,
                    priority: self.options.priority.map(Into::into),
                    ..Default::default()
                }
                .into(),
            ),
            user_metadata,
        }
    }
}

/// Options for sending a signal to an external workflow
pub struct SignalWorkflowOptions {
    /// The workflow's id
    pub workflow_id: String,
    /// The particular run to target, or latest if `None`
    pub run_id: Option<String>,
    /// The details of the signal to send
    pub signal: Signal,
}

impl SignalWorkflowOptions {
    /// Create options for sending a signal to another workflow
    pub fn new(
        workflow_id: impl Into<String>,
        run_id: impl Into<String>,
        name: impl Into<String>,
        input: impl IntoIterator<Item = impl Into<Payload>>,
    ) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            run_id: Some(run_id.into()),
            signal: Signal::new(name, input),
        }
    }

    /// Set a header k/v pair attached to the signal
    pub fn with_header(
        &mut self,
        key: impl Into<String>,
        payload: impl Into<Payload>,
    ) -> &mut Self {
        self.signal.data.with_header(key.into(), payload.into());
        self
    }
}

/// Information needed to send a specific signal
pub struct Signal {
    /// The signal name
    pub signal_name: String,
    /// The data the signal carries
    pub data: SignalData,
}

impl Signal {
    /// Create a new signal
    pub fn new(
        name: impl Into<String>,
        input: impl IntoIterator<Item = impl Into<Payload>>,
    ) -> Self {
        Self {
            signal_name: name.into(),
            data: SignalData::new(input),
        }
    }
}

/// Data contained within a signal
#[derive(Default, Debug)]
pub struct SignalData {
    /// The arguments the signal will receive
    pub input: Vec<Payload>,
    /// Metadata attached to the signal
    pub headers: HashMap<String, Payload>,
}

impl SignalData {
    /// Create data for a signal
    pub fn new(input: impl IntoIterator<Item = impl Into<Payload>>) -> Self {
        Self {
            input: input.into_iter().map(Into::into).collect(),
            headers: HashMap::new(),
        }
    }

    /// Set a header k/v pair attached to the signal
    pub fn with_header(
        &mut self,
        key: impl Into<String>,
        payload: impl Into<Payload>,
    ) -> &mut Self {
        self.headers.insert(key.into(), payload.into());
        self
    }
}

/// Options for timer
#[derive(Default, Debug, Clone)]
pub struct TimerOptions {
    /// Duration for the timer
    pub duration: Duration,
    /// Summary of the timer
    pub summary: Option<String>,
}

impl From<Duration> for TimerOptions {
    fn from(duration: Duration) -> Self {
        TimerOptions {
            duration,
            ..Default::default()
        }
    }
}

/// Options for Nexus Operations
#[derive(Default, Debug, Clone)]
pub struct NexusOperationOptions {
    /// Endpoint name, must exist in the endpoint registry or this command will fail.
    pub endpoint: String,
    /// Service name.
    pub service: String,
    /// Operation name.
    pub operation: String,
    /// Input for the operation. The server converts this into Nexus request content and the
    /// appropriate content headers internally when sending the StartOperation request. On the
    /// handler side, if it is also backed by Temporal, the content is transformed back to the
    /// original Payload sent in this command.
    pub input: Option<Payload>,
    /// Schedule-to-close timeout for this operation.
    /// Indicates how long the caller is willing to wait for operation completion.
    /// Calls are retried internally by the server.
    pub schedule_to_close_timeout: Option<Duration>,
    /// Header to attach to the Nexus request.
    /// Users are responsible for encrypting sensitive data in this header as it is stored in
    /// workflow history and transmitted to external services as-is. This is useful for propagating
    /// tracing information. Note these headers are not the same as Temporal headers on internal
    /// activities and child workflows, these are transmitted to Nexus operations that may be
    /// external and are not traditional payloads.
    pub nexus_header: HashMap<String, String>,
    /// Cancellation type for the operation
    pub cancellation_type: Option<NexusOperationCancellationType>,
}

impl IntoWorkflowCommand for NexusOperationOptions {
    fn into_command(self, seq: u32) -> WorkflowCommand {
        WorkflowCommand {
            user_metadata: None,
            variant: Some(
                ScheduleNexusOperation {
                    seq,
                    endpoint: self.endpoint,
                    service: self.service,
                    operation: self.operation,
                    input: self.input,
                    schedule_to_close_timeout: self
                        .schedule_to_close_timeout
                        .and_then(|t| t.try_into().ok()),
                    nexus_header: self.nexus_header,
                    cancellation_type: self
                        .cancellation_type
                        .unwrap_or(NexusOperationCancellationType::WaitCancellationCompleted)
                        .into(),
                }
                .into(),
            ),
        }
    }
}
```
### `sdk/src/workflow_context.rs`
```rs
mod options;

pub use options::{
    ActivityOptions, ChildWorkflowOptions, LocalActivityOptions, NexusOperationOptions, Signal,
    SignalData, SignalWorkflowOptions, TimerOptions,
};

use crate::{
    CancelExternalWfResult, CancellableID, CancellableIDWithReason, CommandCreateRequest,
    CommandSubscribeChildWorkflowCompletion, IntoUpdateHandlerFunc, IntoUpdateValidatorFunc,
    NexusStartResult, RustWfCmd, SignalExternalWfResult, SupportsCancelReason, TimerResult,
    UnblockEvent, Unblockable, UpdateFunctions, workflow_context::options::IntoWorkflowCommand,
};
use futures_util::{FutureExt, Stream, StreamExt, future::Shared, task::Context};
use parking_lot::{RwLock, RwLockReadGuard};
use std::{
    collections::HashMap,
    future,
    future::Future,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    task::Poll,
    time::{Duration, SystemTime},
};
use temporal_sdk_core_api::worker::WorkerDeploymentVersion;
use temporal_sdk_core_protos::{
    coresdk::{
        activity_result::{ActivityResolution, activity_resolution},
        child_workflow::ChildWorkflowResult,
        common::NamespacedWorkflowExecution,
        nexus::NexusOperationResult,
        workflow_activation::{
            InitializeWorkflow,
            resolve_child_workflow_execution_start::Status as ChildWorkflowStartStatus,
        },
        workflow_commands::{
            CancelChildWorkflowExecution, ModifyWorkflowProperties,
            RequestCancelExternalWorkflowExecution, SetPatchMarker,
            SignalExternalWorkflowExecution, StartTimer, UpsertWorkflowSearchAttributes,
            WorkflowCommand, signal_external_workflow_execution as sig_we, workflow_command,
        },
    },
    temporal::api::{
        common::v1::{Memo, Payload, SearchAttributes},
        sdk::v1::UserMetadata,
    },
};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Used within workflows to issue commands, get info, etc.
#[derive(Clone)]
pub struct WfContext {
    namespace: String,
    task_queue: String,
    inital_information: Arc<InitializeWorkflow>,

    chan: Sender<RustWfCmd>,
    am_cancelled: watch::Receiver<Option<String>>,
    pub(crate) shared: Arc<RwLock<WfContextSharedData>>,

    seq_nums: Arc<RwLock<WfCtxProtectedDat>>,
}

// TODO: Dataconverter type interface to replace Payloads here. Possibly just use serde
//    traits.
impl WfContext {
    /// Create a new wf context, returning the context itself and a receiver which outputs commands
    /// sent from the workflow.
    pub(super) fn new(
        namespace: String,
        task_queue: String,
        init_workflow_job: InitializeWorkflow,
        am_cancelled: watch::Receiver<Option<String>>,
    ) -> (Self, Receiver<RustWfCmd>) {
        // The receiving side is non-async
        let (chan, rx) = std::sync::mpsc::channel();
        (
            Self {
                namespace,
                task_queue,
                shared: Arc::new(RwLock::new(WfContextSharedData {
                    random_seed: init_workflow_job.randomness_seed,
                    search_attributes: init_workflow_job
                        .search_attributes
                        .clone()
                        .unwrap_or_default(),
                    ..Default::default()
                })),
                inital_information: Arc::new(init_workflow_job),
                chan,
                am_cancelled,
                seq_nums: Arc::new(RwLock::new(WfCtxProtectedDat {
                    next_timer_sequence_number: 1,
                    next_activity_sequence_number: 1,
                    next_child_workflow_sequence_number: 1,
                    next_cancel_external_wf_sequence_number: 1,
                    next_signal_external_wf_sequence_number: 1,
                    next_nexus_op_sequence_number: 1,
                })),
            },
            rx,
        )
    }

    /// Return the namespace the workflow is executing in
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Return the task queue the workflow is executing in
    pub fn task_queue(&self) -> &str {
        &self.task_queue
    }

    /// Get the arguments provided to the workflow upon execution start
    pub fn get_args(&self) -> &[Payload] {
        self.inital_information.arguments.as_slice()
    }

    /// Return the current time according to the workflow (which is not wall-clock time).
    pub fn workflow_time(&self) -> Option<SystemTime> {
        self.shared.read().wf_time
    }

    /// Return the length of history so far at this point in the workflow
    pub fn history_length(&self) -> u32 {
        self.shared.read().history_length
    }

    /// Return the deployment version, if any,  as it was when this point in the workflow was first
    /// reached. If this code is being executed for the first time, return this Worker's deployment
    /// version if it has one.
    pub fn current_deployment_version(&self) -> Option<WorkerDeploymentVersion> {
        self.shared.read().current_deployment_version.clone()
    }

    /// Return current values for workflow search attributes
    pub fn search_attributes(&self) -> impl Deref<Target = SearchAttributes> + '_ {
        RwLockReadGuard::map(self.shared.read(), |s| &s.search_attributes)
    }

    /// Return the workflow's randomness seed
    pub fn random_seed(&self) -> u64 {
        self.shared.read().random_seed
    }

    /// Returns true if the current workflow task is happening under replay
    pub fn is_replaying(&self) -> bool {
        self.shared.read().is_replaying
    }

    /// Return various information that the workflow was initialized with. Will eventually become
    /// a proper non-proto workflow info struct.
    pub fn workflow_initial_info(&self) -> &InitializeWorkflow {
        &self.inital_information
    }

    /// A future that resolves if/when the workflow is cancelled, with the user provided cause
    pub async fn cancelled(&self) -> String {
        if let Some(s) = self.am_cancelled.borrow().as_ref() {
            return s.clone();
        }
        self.am_cancelled
            .clone()
            .changed()
            .await
            .expect("Cancelled send half not dropped");
        self.am_cancelled
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or_default()
    }

    /// Request to create a timer
    pub fn timer<T: Into<TimerOptions>>(&self, opts: T) -> impl CancellableFuture<TimerResult> {
        let opts: TimerOptions = opts.into();
        let seq = self.seq_nums.write().next_timer_seq();
        let (cmd, unblocker) = CancellableWFCommandFut::new(CancellableID::Timer(seq));
        self.send(
            CommandCreateRequest {
                cmd: WorkflowCommand {
                    variant: Some(
                        StartTimer {
                            seq,
                            start_to_fire_timeout: Some(
                                opts.duration
                                    .try_into()
                                    .expect("Durations must fit into 64 bits"),
                            ),
                        }
                        .into(),
                    ),
                    user_metadata: Some(UserMetadata {
                        summary: opts.summary.map(|x| x.as_bytes().into()),
                        details: None,
                    }),
                },
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Request to run an activity
    pub fn activity(
        &self,
        mut opts: ActivityOptions,
    ) -> impl CancellableFuture<ActivityResolution> {
        if opts.task_queue.is_none() {
            opts.task_queue = Some(self.task_queue.clone());
        }
        let seq = self.seq_nums.write().next_activity_seq();
        let (cmd, unblocker) = CancellableWFCommandFut::new(CancellableID::Activity(seq));
        self.send(
            CommandCreateRequest {
                cmd: opts.into_command(seq),
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Request to run a local activity
    pub fn local_activity(
        &self,
        opts: LocalActivityOptions,
    ) -> impl CancellableFuture<ActivityResolution> + '_ {
        LATimerBackoffFut::new(opts, self)
    }

    /// Request to run a local activity with no implementation of timer-backoff based retrying.
    fn local_activity_no_timer_retry(
        &self,
        opts: LocalActivityOptions,
    ) -> impl CancellableFuture<ActivityResolution> {
        let seq = self.seq_nums.write().next_activity_seq();
        let (cmd, unblocker) = CancellableWFCommandFut::new(CancellableID::LocalActivity(seq));
        self.send(
            CommandCreateRequest {
                cmd: opts.into_command(seq),
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Creates a child workflow stub with the provided options
    pub fn child_workflow(&self, opts: ChildWorkflowOptions) -> ChildWorkflow {
        ChildWorkflow { opts }
    }

    /// Check (or record) that this workflow history was created with the provided patch
    pub fn patched(&self, patch_id: &str) -> bool {
        self.patch_impl(patch_id, false)
    }

    /// Record that this workflow history was created with the provided patch, and it is being
    /// phased out.
    pub fn deprecate_patch(&self, patch_id: &str) -> bool {
        self.patch_impl(patch_id, true)
    }

    fn patch_impl(&self, patch_id: &str, deprecated: bool) -> bool {
        self.send(
            workflow_command::Variant::SetPatchMarker(SetPatchMarker {
                patch_id: patch_id.to_string(),
                deprecated,
            })
            .into(),
        );
        // See if we already know about the status of this change
        if let Some(present) = self.shared.read().changes.get(patch_id) {
            return *present;
        }

        // If we don't already know about the change, that means there is no marker in history,
        // and we should return false if we are replaying
        let res = !self.shared.read().is_replaying;

        self.shared
            .write()
            .changes
            .insert(patch_id.to_string(), res);

        res
    }

    /// Send a signal to an external workflow. May resolve as a failure if the signal didn't work
    /// or was cancelled.
    pub fn signal_workflow(
        &self,
        opts: impl Into<SignalWorkflowOptions>,
    ) -> impl CancellableFuture<SignalExternalWfResult> {
        let options: SignalWorkflowOptions = opts.into();
        let target = sig_we::Target::WorkflowExecution(NamespacedWorkflowExecution {
            namespace: self.namespace.clone(),
            workflow_id: options.workflow_id,
            run_id: options.run_id.unwrap_or_default(),
        });
        self.send_signal_wf(target, options.signal)
    }

    /// Add or create a set of search attributes
    pub fn upsert_search_attributes(&self, attr_iter: impl IntoIterator<Item = (String, Payload)>) {
        self.send(RustWfCmd::NewNonblockingCmd(
            workflow_command::Variant::UpsertWorkflowSearchAttributes(
                UpsertWorkflowSearchAttributes {
                    search_attributes: HashMap::from_iter(attr_iter),
                },
            ),
        ))
    }

    /// Add or create a set of search attributes
    pub fn upsert_memo(&self, attr_iter: impl IntoIterator<Item = (String, Payload)>) {
        self.send(RustWfCmd::NewNonblockingCmd(
            workflow_command::Variant::ModifyWorkflowProperties(ModifyWorkflowProperties {
                upserted_memo: Some(Memo {
                    fields: HashMap::from_iter(attr_iter),
                }),
            }),
        ))
    }

    /// Return a stream that produces values when the named signal is sent to this workflow
    pub fn make_signal_channel(&self, signal_name: impl Into<String>) -> DrainableSignalStream {
        let (tx, rx) = mpsc::unbounded_channel();
        self.send(RustWfCmd::SubscribeSignal(signal_name.into(), tx));
        DrainableSignalStream(UnboundedReceiverStream::new(rx))
    }

    /// Force a workflow task failure (EX: in order to retry on non-sticky queue)
    pub fn force_task_fail(&self, with: anyhow::Error) {
        self.send(with.into());
    }

    /// Request the cancellation of an external workflow. May resolve as a failure if the workflow
    /// was not found or the cancel was otherwise unsendable.
    pub fn cancel_external(
        &self,
        target: NamespacedWorkflowExecution,
        reason: String,
    ) -> impl Future<Output = CancelExternalWfResult> {
        let seq = self.seq_nums.write().next_cancel_external_wf_seq();
        let (cmd, unblocker) = WFCommandFut::new();
        self.send(
            CommandCreateRequest {
                cmd: WorkflowCommand {
                    variant: Some(
                        RequestCancelExternalWorkflowExecution {
                            seq,
                            workflow_execution: Some(target),
                            reason,
                        }
                        .into(),
                    ),
                    user_metadata: None,
                },
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Register an update handler by providing the handler name, a validator function, and an
    /// update handler. The validator must not mutate workflow state and is synchronous. The handler
    /// may mutate workflow state (though, that's annoying right now in the prototype) and is async.
    ///
    /// Note that if you want a validator that always passes, you will likely need to provide type
    /// annotations to make the compiler happy, like: `|_: &_, _: T| Ok(())`
    pub fn update_handler<Arg, Res>(
        &self,
        name: impl Into<String>,
        validator: impl IntoUpdateValidatorFunc<Arg>,
        handler: impl IntoUpdateHandlerFunc<Arg, Res>,
    ) {
        self.send(RustWfCmd::RegisterUpdate(
            name.into(),
            UpdateFunctions::new(validator, handler),
        ))
    }

    /// Start a nexus operation
    pub fn start_nexus_operation(
        &self,
        opts: NexusOperationOptions,
    ) -> impl CancellableFuture<NexusStartResult> {
        let seq = self.seq_nums.write().next_nexus_op_seq();
        let (result_future, unblocker) = WFCommandFut::new();
        self.send(RustWfCmd::SubscribeNexusOperationCompletion { seq, unblocker });
        let (cmd, unblocker) = CancellableWFCommandFut::new_with_dat(
            CancellableID::NexusOp(seq),
            NexusUnblockData {
                result_future: result_future.shared(),
                schedule_seq: seq,
            },
        );
        self.send(
            CommandCreateRequest {
                cmd: opts.into_command(seq),
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Wait for some condition to become true, yielding the workflow if it is not.
    pub fn wait_condition(&self, mut condition: impl FnMut() -> bool) -> impl Future<Output = ()> {
        future::poll_fn(move |_cx: &mut Context<'_>| {
            if condition() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }

    /// Buffer a command to be sent in the activation reply
    pub(crate) fn send(&self, c: RustWfCmd) {
        self.chan.send(c).expect("command channel intact");
    }

    fn send_signal_wf(
        &self,
        target: sig_we::Target,
        signal: Signal,
    ) -> impl CancellableFuture<SignalExternalWfResult> {
        let seq = self.seq_nums.write().next_signal_external_wf_seq();
        let (cmd, unblocker) =
            CancellableWFCommandFut::new(CancellableID::SignalExternalWorkflow(seq));
        self.send(
            CommandCreateRequest {
                cmd: WorkflowCommand {
                    variant: Some(
                        SignalExternalWorkflowExecution {
                            seq,
                            signal_name: signal.signal_name,
                            args: signal.data.input,
                            target: Some(target),
                            headers: signal.data.headers,
                        }
                        .into(),
                    ),
                    user_metadata: None,
                },
                unblocker,
            }
            .into(),
        );
        cmd
    }

    /// Cancel any cancellable operation by ID
    fn cancel(&self, cancellable_id: CancellableID) {
        self.send(RustWfCmd::Cancel(cancellable_id));
    }
}

struct WfCtxProtectedDat {
    next_timer_sequence_number: u32,
    next_activity_sequence_number: u32,
    next_child_workflow_sequence_number: u32,
    next_cancel_external_wf_sequence_number: u32,
    next_signal_external_wf_sequence_number: u32,
    next_nexus_op_sequence_number: u32,
}

impl WfCtxProtectedDat {
    fn next_timer_seq(&mut self) -> u32 {
        let seq = self.next_timer_sequence_number;
        self.next_timer_sequence_number += 1;
        seq
    }
    fn next_activity_seq(&mut self) -> u32 {
        let seq = self.next_activity_sequence_number;
        self.next_activity_sequence_number += 1;
        seq
    }
    fn next_child_workflow_seq(&mut self) -> u32 {
        let seq = self.next_child_workflow_sequence_number;
        self.next_child_workflow_sequence_number += 1;
        seq
    }
    fn next_cancel_external_wf_seq(&mut self) -> u32 {
        let seq = self.next_cancel_external_wf_sequence_number;
        self.next_cancel_external_wf_sequence_number += 1;
        seq
    }
    fn next_signal_external_wf_seq(&mut self) -> u32 {
        let seq = self.next_signal_external_wf_sequence_number;
        self.next_signal_external_wf_sequence_number += 1;
        seq
    }
    fn next_nexus_op_seq(&mut self) -> u32 {
        let seq = self.next_nexus_op_sequence_number;
        self.next_nexus_op_sequence_number += 1;
        seq
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WfContextSharedData {
    /// Maps change ids -> resolved status
    pub(crate) changes: HashMap<String, bool>,
    pub(crate) is_replaying: bool,
    pub(crate) wf_time: Option<SystemTime>,
    pub(crate) history_length: u32,
    pub(crate) current_deployment_version: Option<WorkerDeploymentVersion>,
    pub(crate) search_attributes: SearchAttributes,
    pub(crate) random_seed: u64,
}

/// Helper Wrapper that can drain the channel into a Vec<SignalData> in a blocking way.  Useful
/// for making sure channels are empty before ContinueAsNew-ing a workflow
pub struct DrainableSignalStream(UnboundedReceiverStream<SignalData>);

impl DrainableSignalStream {
    pub fn drain_all(self) -> Vec<SignalData> {
        let mut receiver = self.0.into_inner();
        let mut signals = vec![];
        while let Ok(s) = receiver.try_recv() {
            signals.push(s);
        }
        signals
    }

    pub fn drain_ready(&mut self) -> Vec<SignalData> {
        let mut signals = vec![];
        while let Some(s) = self.0.next().now_or_never().flatten() {
            signals.push(s);
        }
        signals
    }
}

impl Stream for DrainableSignalStream {
    type Item = SignalData;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

/// A Future that can be cancelled.
/// Used in the prototype SDK for cancelling operations like timers and activities.
pub trait CancellableFuture<T>: Future<Output = T> {
    /// Cancel this Future
    fn cancel(&self, cx: &WfContext);
}

/// A Future that can be cancelled with a reason
pub trait CancellableFutureWithReason<T>: CancellableFuture<T> {
    /// Cancel this Future with a reason
    fn cancel_with_reason(&self, cx: &WfContext, reason: String);
}

struct WFCommandFut<T, D> {
    _unused: PhantomData<T>,
    result_rx: oneshot::Receiver<UnblockEvent>,
    other_dat: Option<D>,
}
impl<T> WFCommandFut<T, ()> {
    fn new() -> (Self, oneshot::Sender<UnblockEvent>) {
        Self::new_with_dat(())
    }
}

impl<T, D> WFCommandFut<T, D> {
    fn new_with_dat(other_dat: D) -> (Self, oneshot::Sender<UnblockEvent>) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                _unused: PhantomData,
                result_rx: rx,
                other_dat: Some(other_dat),
            },
            tx,
        )
    }
}

impl<T, D> Unpin for WFCommandFut<T, D> where T: Unblockable<OtherDat = D> {}
impl<T, D> Future for WFCommandFut<T, D>
where
    T: Unblockable<OtherDat = D>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.result_rx.poll_unpin(cx).map(|x| {
            // SAFETY: Because we can only enter this section once the future has resolved, we
            // know it will never be polled again, therefore consuming the option is OK.
            let od = self
                .other_dat
                .take()
                .expect("Other data must exist when resolving command future");
            Unblockable::unblock(x.unwrap(), od)
        })
    }
}

struct CancellableWFCommandFut<T, D, ID = CancellableID> {
    cmd_fut: WFCommandFut<T, D>,
    cancellable_id: ID,
}
impl<T, ID> CancellableWFCommandFut<T, (), ID> {
    fn new(cancellable_id: ID) -> (Self, oneshot::Sender<UnblockEvent>) {
        Self::new_with_dat(cancellable_id, ())
    }
}
impl<T, D, ID> CancellableWFCommandFut<T, D, ID> {
    fn new_with_dat(cancellable_id: ID, other_dat: D) -> (Self, oneshot::Sender<UnblockEvent>) {
        let (cmd_fut, sender) = WFCommandFut::new_with_dat(other_dat);
        (
            Self {
                cmd_fut,
                cancellable_id,
            },
            sender,
        )
    }
}
impl<T, D, ID> Unpin for CancellableWFCommandFut<T, D, ID> where T: Unblockable<OtherDat = D> {}
impl<T, D, ID> Future for CancellableWFCommandFut<T, D, ID>
where
    T: Unblockable<OtherDat = D>,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.cmd_fut.poll_unpin(cx)
    }
}

impl<T, D, ID> CancellableFuture<T> for CancellableWFCommandFut<T, D, ID>
where
    T: Unblockable<OtherDat = D>,
    ID: Clone + Into<CancellableID>,
{
    fn cancel(&self, cx: &WfContext) {
        cx.cancel(self.cancellable_id.clone().into());
    }
}
impl<T, D> CancellableFutureWithReason<T> for CancellableWFCommandFut<T, D, CancellableIDWithReason>
where
    T: Unblockable<OtherDat = D>,
{
    fn cancel_with_reason(&self, cx: &WfContext, reason: String) {
        let new_id = self.cancellable_id.clone().with_reason(reason);
        cx.cancel(new_id);
    }
}

struct LATimerBackoffFut<'a> {
    la_opts: LocalActivityOptions,
    current_fut: Pin<Box<dyn CancellableFuture<ActivityResolution> + Send + Unpin + 'a>>,
    timer_fut: Option<Pin<Box<dyn CancellableFuture<TimerResult> + Send + Unpin + 'a>>>,
    ctx: &'a WfContext,
    next_attempt: u32,
    next_sched_time: Option<prost_types::Timestamp>,
    did_cancel: AtomicBool,
}
impl<'a> LATimerBackoffFut<'a> {
    pub(crate) fn new(opts: LocalActivityOptions, ctx: &'a WfContext) -> Self {
        Self {
            la_opts: opts.clone(),
            current_fut: Box::pin(ctx.local_activity_no_timer_retry(opts)),
            timer_fut: None,
            ctx,
            next_attempt: 1,
            next_sched_time: None,
            did_cancel: AtomicBool::new(false),
        }
    }
}
impl Unpin for LATimerBackoffFut<'_> {}
impl Future for LATimerBackoffFut<'_> {
    type Output = ActivityResolution;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // If the timer exists, wait for it first
        if let Some(tf) = self.timer_fut.as_mut() {
            return match tf.poll_unpin(cx) {
                Poll::Ready(tr) => {
                    self.timer_fut = None;
                    // Schedule next LA if this timer wasn't cancelled
                    if let TimerResult::Fired = tr {
                        let mut opts = self.la_opts.clone();
                        opts.attempt = Some(self.next_attempt);
                        opts.original_schedule_time
                            .clone_from(&self.next_sched_time);
                        self.current_fut = Box::pin(self.ctx.local_activity_no_timer_retry(opts));
                        Poll::Pending
                    } else {
                        Poll::Ready(ActivityResolution {
                            status: Some(
                                activity_resolution::Status::Cancelled(Default::default()),
                            ),
                        })
                    }
                }
                Poll::Pending => Poll::Pending,
            };
        }
        let poll_res = self.current_fut.poll_unpin(cx);
        if let Poll::Ready(ref r) = poll_res
            && let Some(activity_resolution::Status::Backoff(b)) = r.status.as_ref()
        {
            // If we've already said we want to cancel, don't schedule the backoff timer. Just
            // return cancel status. This can happen if cancel comes after the LA says it wants
            // to back off but before we have scheduled the timer.
            if self.did_cancel.load(Ordering::Acquire) {
                return Poll::Ready(ActivityResolution {
                    status: Some(activity_resolution::Status::Cancelled(Default::default())),
                });
            }

            let timer_f = self.ctx.timer::<Duration>(
                b.backoff_duration
                    .expect("Duration is set")
                    .try_into()
                    .expect("duration converts ok"),
            );
            self.timer_fut = Some(Box::pin(timer_f));
            self.next_attempt = b.attempt;
            self.next_sched_time.clone_from(&b.original_schedule_time);
            return Poll::Pending;
        }
        poll_res
    }
}
impl CancellableFuture<ActivityResolution> for LATimerBackoffFut<'_> {
    fn cancel(&self, ctx: &WfContext) {
        self.did_cancel.store(true, Ordering::Release);
        if let Some(tf) = self.timer_fut.as_ref() {
            tf.cancel(ctx);
        }
        self.current_fut.cancel(ctx);
    }
}

/// A stub representing an unstarted child workflow.
#[derive(Default, Debug, Clone)]
pub struct ChildWorkflow {
    opts: ChildWorkflowOptions,
}

pub(crate) struct ChildWfCommon {
    workflow_id: String,
    result_future: CancellableWFCommandFut<ChildWorkflowResult, (), CancellableIDWithReason>,
}

/// Child workflow in pending state
pub struct PendingChildWorkflow {
    /// The status of the child workflow start
    pub status: ChildWorkflowStartStatus,
    pub(crate) common: ChildWfCommon,
}

impl PendingChildWorkflow {
    /// Returns `None` if the child did not start successfully. The returned [StartedChildWorkflow]
    /// can be used to wait on, signal, or cancel the child workflow.
    pub fn into_started(self) -> Option<StartedChildWorkflow> {
        match self.status {
            ChildWorkflowStartStatus::Succeeded(s) => Some(StartedChildWorkflow {
                run_id: s.run_id,
                common: self.common,
            }),
            _ => None,
        }
    }
}

/// Child workflow in started state
pub struct StartedChildWorkflow {
    /// Run ID of the child workflow
    pub run_id: String,
    common: ChildWfCommon,
}

impl ChildWorkflow {
    /// Start the child workflow, the returned Future is cancellable.
    pub fn start(self, cx: &WfContext) -> impl CancellableFutureWithReason<PendingChildWorkflow> {
        let child_seq = cx.seq_nums.write().next_child_workflow_seq();
        // Immediately create the command/future for the result, otherwise if the user does
        // not await the result until *after* we receive an activation for it, there will be nothing
        // to match when unblocking.
        let cancel_seq = cx.seq_nums.write().next_cancel_external_wf_seq();
        let (result_cmd, unblocker) =
            CancellableWFCommandFut::new(CancellableIDWithReason::ExternalWorkflow {
                seqnum: cancel_seq,
                execution: NamespacedWorkflowExecution {
                    workflow_id: self.opts.workflow_id.clone(),
                    ..Default::default()
                },
            });
        cx.send(
            CommandSubscribeChildWorkflowCompletion {
                seq: child_seq,
                unblocker,
            }
            .into(),
        );

        let common = ChildWfCommon {
            workflow_id: self.opts.workflow_id.clone(),
            result_future: result_cmd,
        };

        let (cmd, unblocker) = CancellableWFCommandFut::new_with_dat(
            CancellableIDWithReason::ChildWorkflow { seqnum: child_seq },
            common,
        );
        cx.send(
            CommandCreateRequest {
                cmd: self.opts.into_command(child_seq),
                unblocker,
            }
            .into(),
        );

        cmd
    }
}

impl StartedChildWorkflow {
    /// Consumes self and returns a future that will wait until completion of this child workflow
    /// execution
    pub fn result(self) -> impl CancellableFutureWithReason<ChildWorkflowResult> {
        self.common.result_future
    }

    /// Cancel the child workflow
    pub fn cancel(&self, cx: &WfContext, reason: String) {
        cx.send(RustWfCmd::NewNonblockingCmd(
            CancelChildWorkflowExecution {
                child_workflow_seq: self.common.result_future.cancellable_id.seq_num(),
                reason,
            }
            .into(),
        ));
    }

    /// Signal the child workflow
    pub fn signal<'a, S: Into<Signal>>(
        &self,
        cx: &'a WfContext,
        data: S,
    ) -> impl CancellableFuture<SignalExternalWfResult> + use<'a, S> {
        let target = sig_we::Target::ChildWorkflowId(self.common.workflow_id.clone());
        cx.send_signal_wf(target, data.into())
    }
}

#[derive(derive_more::Debug)]
#[debug("StartedNexusOperation{{ operation_token: {operation_token:?} }}")]
pub struct StartedNexusOperation {
    /// The operation token, if the operation started asynchronously
    pub operation_token: Option<String>,
    pub(crate) unblock_dat: NexusUnblockData,
}

pub(crate) struct NexusUnblockData {
    result_future: Shared<WFCommandFut<NexusOperationResult, ()>>,
    schedule_seq: u32,
}

impl StartedNexusOperation {
    pub async fn result(&self) -> NexusOperationResult {
        self.unblock_dat.result_future.clone().await
    }

    pub fn cancel(&self, cx: &WfContext) {
        cx.cancel(CancellableID::NexusOp(self.unblock_dat.schedule_seq));
    }
}
```
### `sdk/src/workflow_future.rs`
```rs
use crate::{
    CancellableID, RustWfCmd, SignalData, TimerResult, UnblockEvent, UpdateContext,
    UpdateFunctions, UpdateInfo, WfContext, WfExitValue, WorkflowFunction, WorkflowResult,
    panic_formatter,
};
use anyhow::{Context as AnyhowContext, Error, anyhow, bail};
use futures_util::{FutureExt, future::BoxFuture};
use std::{
    collections::{HashMap, hash_map::Entry},
    future::Future,
    panic,
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::mpsc::Receiver,
    task::{Context, Poll},
};
use temporal_sdk_core_protos::{
    coresdk::{
        workflow_activation::{
            FireTimer, InitializeWorkflow, NotifyHasPatch, ResolveActivity,
            ResolveChildWorkflowExecution, ResolveChildWorkflowExecutionStart, WorkflowActivation,
            WorkflowActivationJob, workflow_activation_job::Variant,
        },
        workflow_commands::{
            CancelChildWorkflowExecution, CancelSignalWorkflow, CancelTimer,
            CancelWorkflowExecution, CompleteWorkflowExecution, FailWorkflowExecution,
            RequestCancelActivity, RequestCancelExternalWorkflowExecution,
            RequestCancelLocalActivity, RequestCancelNexusOperation, ScheduleActivity,
            ScheduleLocalActivity, StartTimer, UpdateResponse, WorkflowCommand, update_response,
            workflow_command,
        },
        workflow_completion,
        workflow_completion::{WorkflowActivationCompletion, workflow_activation_completion},
    },
    temporal::api::{common::v1::Payload, enums::v1::VersioningBehavior, failure::v1::Failure},
    utilities::TryIntoOrNone,
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    oneshot, watch,
};
use tracing::Instrument;

impl WorkflowFunction {
    /// Start a workflow function, returning a future that will resolve when the workflow does,
    /// and a channel that can be used to send it activations.
    pub(crate) fn start_workflow(
        &self,
        namespace: String,
        task_queue: String,
        init_workflow_job: InitializeWorkflow,
        outgoing_completions: UnboundedSender<WorkflowActivationCompletion>,
    ) -> (
        impl Future<Output = WorkflowResult<Payload>> + use<>,
        UnboundedSender<WorkflowActivation>,
    ) {
        let (cancel_tx, cancel_rx) = watch::channel(None);
        let span = info_span!(
            "RunWorkflow",
            "otel.name" = format!("RunWorkflow:{}", &init_workflow_job.workflow_type),
            "otel.kind" = "server"
        );
        let (wf_context, cmd_receiver) =
            WfContext::new(namespace, task_queue, init_workflow_job, cancel_rx);
        let (tx, incoming_activations) = unbounded_channel();
        let inner_fut = (self.wf_func)(wf_context.clone()).instrument(span);
        (
            WorkflowFuture {
                wf_ctx: wf_context,
                // We need to mark the workflow future as unconstrained, otherwise Tokio will impose
                // an artificial limit on how many commands we can unblock in one poll round.
                // TODO: Now we *need* deadlock detection or we could hose the whole system
                inner: tokio::task::unconstrained(inner_fut).fuse().boxed(),
                incoming_commands: cmd_receiver,
                outgoing_completions,
                incoming_activations,
                command_status: Default::default(),
                cancel_sender: cancel_tx,
                sig_chans: Default::default(),
                updates: Default::default(),
                update_futures: Default::default(),
            },
            tx,
        )
    }
}

struct WFCommandFutInfo {
    unblocker: oneshot::Sender<UnblockEvent>,
}

// Allows the workflow to receive signals even though the signal handler may not yet be registered.
// TODO: Either make this go away by requiring all signals to be registered up-front in a more
//   production-ready SDK design, or if desired to allow dynamic signal registration, prevent this
//   from growing unbounded if being sent lots of unhandled signals.
enum SigChanOrBuffer {
    Chan(UnboundedSender<SignalData>),
    Buffer(Vec<SignalData>),
}

pub(crate) struct WorkflowFuture {
    /// Future produced by calling the workflow function
    inner: BoxFuture<'static, WorkflowResult<Payload>>,
    /// Commands produced inside user's wf code
    incoming_commands: Receiver<RustWfCmd>,
    /// Once blocked or the workflow has finished or errored out, the result is sent here
    outgoing_completions: UnboundedSender<WorkflowActivationCompletion>,
    /// Activations from core
    incoming_activations: UnboundedReceiver<WorkflowActivation>,
    /// Commands by ID -> blocked status
    command_status: HashMap<CommandID, WFCommandFutInfo>,
    /// Use to notify workflow code of cancellation
    cancel_sender: watch::Sender<Option<String>>,
    /// Copy of the workflow context
    wf_ctx: WfContext,
    /// Maps signal IDs to channels to send down when they are signaled
    sig_chans: HashMap<String, SigChanOrBuffer>,
    /// Maps update handlers by name to implementations
    updates: HashMap<String, UpdateFunctions>,
    /// Stores in-progress update futures
    update_futures: Vec<(String, BoxFuture<'static, Result<Payload, Error>>)>,
}

impl WorkflowFuture {
    fn unblock(&mut self, event: UnblockEvent) -> Result<(), Error> {
        let cmd_id = match event {
            UnblockEvent::Timer(seq, _) => CommandID::Timer(seq),
            UnblockEvent::Activity(seq, _) => CommandID::Activity(seq),
            UnblockEvent::WorkflowStart(seq, _) => CommandID::ChildWorkflowStart(seq),
            UnblockEvent::WorkflowComplete(seq, _) => CommandID::ChildWorkflowComplete(seq),
            UnblockEvent::SignalExternal(seq, _) => CommandID::SignalExternal(seq),
            UnblockEvent::CancelExternal(seq, _) => CommandID::CancelExternal(seq),
            UnblockEvent::NexusOperationStart(seq, _) => CommandID::NexusOpStart(seq),
            UnblockEvent::NexusOperationComplete(seq, _) => CommandID::NexusOpComplete(seq),
        };
        let unblocker = self.command_status.remove(&cmd_id);
        let _ = unblocker
            .ok_or_else(|| anyhow!("Command {:?} not found to unblock!", cmd_id))?
            .unblocker
            .send(event);
        Ok(())
    }

    fn fail_wft(&self, run_id: String, fail: Error) {
        warn!("Workflow task failed for {}: {}", run_id, fail);
        self.outgoing_completions
            .send(WorkflowActivationCompletion::fail(
                run_id,
                fail.into(),
                None,
            ))
            .expect("Completion channel intact");
    }

    fn send_completion(&self, run_id: String, activation_cmds: Vec<WorkflowCommand>) {
        self.outgoing_completions
            .send(WorkflowActivationCompletion {
                run_id,
                status: Some(workflow_activation_completion::Status::Successful(
                    workflow_completion::Success {
                        commands: activation_cmds,
                        used_internal_flags: vec![],
                        versioning_behavior: VersioningBehavior::Unspecified.into(),
                    },
                )),
            })
            .expect("Completion channel intact");
    }

    /// Handle a particular workflow activation job.
    ///
    /// Returns Ok(true) if the workflow should be evicted. Returns an error in the event that
    /// the workflow task should be failed.
    ///
    /// Panics if internal assumptions are violated
    fn handle_job(
        &mut self,
        variant: Option<Variant>,
        outgoing_cmds: &mut Vec<WorkflowCommand>,
    ) -> Result<(), Error> {
        if let Some(v) = variant {
            match v {
                Variant::InitializeWorkflow(_) => {
                    // Don't do anything in here. Init workflow is looked at earlier, before
                    // jobs are handled, and may have information taken out of it to avoid clones.
                }
                Variant::FireTimer(FireTimer { seq }) => {
                    self.unblock(UnblockEvent::Timer(seq, TimerResult::Fired))?
                }
                Variant::ResolveActivity(ResolveActivity { seq, result, .. }) => {
                    self.unblock(UnblockEvent::Activity(
                        seq,
                        Box::new(result.context("Activity must have result")?),
                    ))?;
                }
                Variant::ResolveChildWorkflowExecutionStart(
                    ResolveChildWorkflowExecutionStart { seq, status },
                ) => self.unblock(UnblockEvent::WorkflowStart(
                    seq,
                    Box::new(status.context("Workflow start must have status")?),
                ))?,
                Variant::ResolveChildWorkflowExecution(ResolveChildWorkflowExecution {
                    seq,
                    result,
                }) => self.unblock(UnblockEvent::WorkflowComplete(
                    seq,
                    Box::new(result.context("Child Workflow execution must have a result")?),
                ))?,
                Variant::UpdateRandomSeed(rs) => {
                    self.wf_ctx.shared.write().random_seed = rs.randomness_seed;
                }
                Variant::QueryWorkflow(q) => {
                    error!(
                        "Queries are not implemented in the Rust SDK. Got query '{}'",
                        q.query_type
                    );
                }
                Variant::CancelWorkflow(c) => {
                    // TODO: Cancel pending futures, etc
                    self.cancel_sender
                        .send(Some(c.reason))
                        .expect("Cancel rx not dropped");
                }
                Variant::SignalWorkflow(sig) => {
                    let mut dat = SignalData::new(sig.input);
                    dat.headers = sig.headers;
                    match self.sig_chans.entry(sig.signal_name) {
                        Entry::Occupied(mut o) => match o.get_mut() {
                            SigChanOrBuffer::Chan(chan) => {
                                let _ = chan.send(dat);
                            }
                            SigChanOrBuffer::Buffer(buf) => buf.push(dat),
                        },
                        Entry::Vacant(v) => {
                            v.insert(SigChanOrBuffer::Buffer(vec![dat]));
                        }
                    }
                }
                Variant::NotifyHasPatch(NotifyHasPatch { patch_id }) => {
                    self.wf_ctx.shared.write().changes.insert(patch_id, true);
                }
                Variant::ResolveSignalExternalWorkflow(attrs) => {
                    self.unblock(UnblockEvent::SignalExternal(attrs.seq, attrs.failure))?;
                }
                Variant::ResolveRequestCancelExternalWorkflow(attrs) => {
                    self.unblock(UnblockEvent::CancelExternal(attrs.seq, attrs.failure))?;
                }
                Variant::DoUpdate(u) => {
                    if let Some(impls) = self.updates.get_mut(&u.name) {
                        let info = UpdateInfo {
                            update_id: u.id,
                            headers: u.headers,
                        };
                        let defp = Payload::default();
                        let val_res = if u.run_validator {
                            match panic::catch_unwind(AssertUnwindSafe(|| {
                                (impls.validator)(&info, u.input.first().unwrap_or(&defp))
                            })) {
                                Ok(r) => r,
                                Err(e) => {
                                    bail!("Panic in update validator {}", panic_formatter(e));
                                }
                            }
                        } else {
                            Ok(())
                        };
                        match val_res {
                            Ok(_) => {
                                outgoing_cmds.push(
                                    update_response(
                                        u.protocol_instance_id.clone(),
                                        update_response::Response::Accepted(()),
                                    )
                                    .into(),
                                );
                                let handler_fut = (impls.handler)(
                                    UpdateContext {
                                        wf_ctx: self.wf_ctx.clone(),
                                        info,
                                    },
                                    u.input.first().unwrap_or(&defp),
                                );
                                self.update_futures
                                    .push((u.protocol_instance_id, handler_fut));
                            }
                            Err(e) => {
                                outgoing_cmds.push(
                                    update_response(
                                        u.protocol_instance_id,
                                        update_response::Response::Rejected(e.into()),
                                    )
                                    .into(),
                                );
                            }
                        }
                    } else {
                        outgoing_cmds.push(
                            update_response(
                                u.protocol_instance_id,
                                update_response::Response::Rejected(
                                    format!(
                                        "No update handler registered for update name {}",
                                        u.name
                                    )
                                    .into(),
                                ),
                            )
                            .into(),
                        );
                    }
                }
                Variant::ResolveNexusOperationStart(attrs) => {
                    self.unblock(UnblockEvent::NexusOperationStart(
                        attrs.seq,
                        Box::new(
                            attrs
                                .status
                                .context("Nexus operation start must have status")?,
                        ),
                    ))?
                }
                Variant::ResolveNexusOperation(attrs) => {
                    self.unblock(UnblockEvent::NexusOperationComplete(
                        attrs.seq,
                        Box::new(attrs.result.context("Nexus operation must have result")?),
                    ))?
                }
                Variant::RemoveFromCache(_) => {
                    unreachable!("Cache removal should happen higher up");
                }
            }
        } else {
            bail!("Empty activation job variant");
        }

        Ok(())
    }
}

impl Future for WorkflowFuture {
    type Output = WorkflowResult<Payload>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        'activations: loop {
            // WF must always receive an activation first before responding with commands
            let activation = match self.incoming_activations.poll_recv(cx) {
                Poll::Ready(a) => match a {
                    Some(act) => act,
                    None => {
                        return Poll::Ready(Err(anyhow!(
                            "Workflow future's activation channel was lost!"
                        )));
                    }
                },
                Poll::Pending => return Poll::Pending,
            };

            let is_only_eviction = activation.is_only_eviction();
            let run_id = activation.run_id;
            {
                let mut wlock = self.wf_ctx.shared.write();
                wlock.is_replaying = activation.is_replaying;
                wlock.wf_time = activation.timestamp.try_into_or_none();
                wlock.history_length = activation.history_length;
                wlock.current_deployment_version = activation
                    .deployment_version_for_current_task
                    .map(Into::into);
            }

            let mut activation_cmds = vec![];
            // Lame hack to avoid hitting "unregistered" update handlers in a situation where
            // the history has no commands until an update is accepted. Will go away w/ SDK redesign
            if activation
                .jobs
                .iter()
                .any(|j| matches!(j.variant, Some(Variant::InitializeWorkflow(_))))
                && activation.jobs.iter().all(|j| {
                    matches!(
                        j.variant,
                        Some(Variant::InitializeWorkflow(_) | Variant::DoUpdate(_))
                    )
                })
            {
                // Poll the workflow future once to get things registered
                if self.poll_wf_future(cx, &run_id, &mut activation_cmds)? {
                    continue;
                }
            }

            if is_only_eviction {
                // No need to do anything with the workflow code in this case
                self.outgoing_completions
                    .send(WorkflowActivationCompletion::from_cmds(run_id, vec![]))
                    .expect("Completion channel intact");
                return Ok(WfExitValue::Evicted).into();
            }

            for WorkflowActivationJob { variant } in activation.jobs {
                if let Err(e) = self.handle_job(variant, &mut activation_cmds) {
                    self.fail_wft(run_id, e);
                    continue 'activations;
                }
            }

            // Drive update functions
            self.update_futures = std::mem::take(&mut self.update_futures)
                .into_iter()
                .filter_map(
                    |(instance_id, mut update_fut)| match update_fut.poll_unpin(cx) {
                        Poll::Ready(v) => {
                            // Push into the command channel here rather than activation_cmds
                            // directly to avoid completing and update before any final un-awaited
                            // commands started from within it.
                            self.wf_ctx.send(
                                update_response(
                                    instance_id,
                                    match v {
                                        Ok(v) => update_response::Response::Completed(v),
                                        Err(e) => update_response::Response::Rejected(e.into()),
                                    },
                                )
                                .into(),
                            );
                            None
                        }
                        Poll::Pending => Some((instance_id, update_fut)),
                    },
                )
                .collect();

            if self.poll_wf_future(cx, &run_id, &mut activation_cmds)? {
                continue;
            }

            // TODO: deadlock detector
            // Check if there's nothing to unblock and workflow has not completed.
            // This is different from the assertion that was here before that checked that WF did
            // not produce any commands which is completely viable in the case WF is waiting on
            // multiple completions.

            self.send_completion(run_id, activation_cmds);

            // We don't actually return here, since we could be queried after finishing executing,
            // and it allows us to rely on evictions for death and cache management
        }
    }
}

// Separate impl block down here just to keep it close to the future poll implementation which
// it is specific to.
impl WorkflowFuture {
    /// Returns true if the workflow future polling loop should be continued
    fn poll_wf_future(
        &mut self,
        cx: &mut Context,
        run_id: &str,
        activation_cmds: &mut Vec<WorkflowCommand>,
    ) -> Result<bool, Error> {
        // TODO: Make sure this is *actually* safe before un-prototyping rust sdk
        let mut res = match AssertUnwindSafe(&mut self.inner)
            .catch_unwind()
            .poll_unpin(cx)
        {
            Poll::Ready(Err(e)) => {
                let errmsg = format!("Workflow function panicked: {}", panic_formatter(e));
                warn!("{}", errmsg);
                self.outgoing_completions
                    .send(WorkflowActivationCompletion::fail(
                        run_id,
                        Failure {
                            message: errmsg,
                            ..Default::default()
                        },
                        None,
                    ))
                    .expect("Completion channel intact");
                // Loop back up because we're about to get evicted
                return Ok(true);
            }
            Poll::Ready(Ok(r)) => Poll::Ready(r),
            Poll::Pending => Poll::Pending,
        };

        while let Ok(cmd) = self.incoming_commands.try_recv() {
            match cmd {
                RustWfCmd::Cancel(cancellable_id) => {
                    let cmd_variant = match cancellable_id {
                        CancellableID::Timer(seq) => {
                            self.unblock(UnblockEvent::Timer(seq, TimerResult::Cancelled))?;
                            // Re-poll wf future since a timer is now unblocked
                            res = self.inner.poll_unpin(cx);
                            workflow_command::Variant::CancelTimer(CancelTimer { seq })
                        }
                        CancellableID::Activity(seq) => {
                            workflow_command::Variant::RequestCancelActivity(
                                RequestCancelActivity { seq },
                            )
                        }
                        CancellableID::LocalActivity(seq) => {
                            workflow_command::Variant::RequestCancelLocalActivity(
                                RequestCancelLocalActivity { seq },
                            )
                        }
                        CancellableID::ChildWorkflow { seqnum, reason } => {
                            workflow_command::Variant::CancelChildWorkflowExecution(
                                CancelChildWorkflowExecution {
                                    child_workflow_seq: seqnum,
                                    reason,
                                },
                            )
                        }
                        CancellableID::SignalExternalWorkflow(seq) => {
                            workflow_command::Variant::CancelSignalWorkflow(CancelSignalWorkflow {
                                seq,
                            })
                        }
                        CancellableID::ExternalWorkflow {
                            seqnum,
                            execution,
                            reason,
                        } => workflow_command::Variant::RequestCancelExternalWorkflowExecution(
                            RequestCancelExternalWorkflowExecution {
                                seq: seqnum,
                                workflow_execution: Some(execution),
                                reason,
                            },
                        ),
                        CancellableID::NexusOp(seq) => {
                            workflow_command::Variant::RequestCancelNexusOperation(
                                RequestCancelNexusOperation { seq },
                            )
                        }
                    };
                    activation_cmds.push(cmd_variant.into());
                }

                RustWfCmd::NewCmd(cmd) => {
                    let command_id = match cmd.cmd.variant.as_ref().expect("command variant is set")
                    {
                        workflow_command::Variant::StartTimer(StartTimer { seq, .. }) => {
                            CommandID::Timer(*seq)
                        }
                        workflow_command::Variant::ScheduleActivity(ScheduleActivity {
                            seq,
                            ..
                        })
                        | workflow_command::Variant::ScheduleLocalActivity(
                            ScheduleLocalActivity { seq, .. },
                        ) => CommandID::Activity(*seq),
                        workflow_command::Variant::SetPatchMarker(_) => {
                            panic!("Set patch marker should be a nonblocking command")
                        }
                        workflow_command::Variant::StartChildWorkflowExecution(req) => {
                            let seq = req.seq;
                            CommandID::ChildWorkflowStart(seq)
                        }
                        workflow_command::Variant::SignalExternalWorkflowExecution(req) => {
                            CommandID::SignalExternal(req.seq)
                        }
                        workflow_command::Variant::RequestCancelExternalWorkflowExecution(req) => {
                            CommandID::CancelExternal(req.seq)
                        }
                        workflow_command::Variant::ScheduleNexusOperation(req) => {
                            CommandID::NexusOpStart(req.seq)
                        }
                        _ => unimplemented!("Command type not implemented"),
                    };
                    activation_cmds.push(cmd.cmd);

                    self.command_status.insert(
                        command_id,
                        WFCommandFutInfo {
                            unblocker: cmd.unblocker,
                        },
                    );
                }
                RustWfCmd::NewNonblockingCmd(cmd) => {
                    activation_cmds.push(cmd.into());
                }
                RustWfCmd::SubscribeChildWorkflowCompletion(sub) => {
                    self.command_status.insert(
                        CommandID::ChildWorkflowComplete(sub.seq),
                        WFCommandFutInfo {
                            unblocker: sub.unblocker,
                        },
                    );
                }
                RustWfCmd::SubscribeSignal(signame, chan) => {
                    // Deal with any buffered signal inputs for signals that were not yet
                    // registered
                    if let Some(SigChanOrBuffer::Buffer(buf)) = self.sig_chans.remove(&signame) {
                        for input in buf {
                            let _ = chan.send(input);
                        }
                        // Re-poll wf future since signals may be unblocked
                        res = self.inner.poll_unpin(cx);
                    }
                    self.sig_chans.insert(signame, SigChanOrBuffer::Chan(chan));
                }
                RustWfCmd::ForceWFTFailure(err) => {
                    self.fail_wft(run_id.to_string(), err);
                    return Ok(true);
                }
                RustWfCmd::RegisterUpdate(name, impls) => {
                    self.updates.insert(name, impls);
                }
                RustWfCmd::SubscribeNexusOperationCompletion { seq, unblocker } => {
                    self.command_status.insert(
                        CommandID::NexusOpComplete(seq),
                        WFCommandFutInfo { unblocker },
                    );
                }
            }
        }

        if let Poll::Ready(res) = res {
            // TODO: Auto reply with cancel when cancelled (instead of normal exit value)
            let cmd = match res {
                Ok(exit_val) => match exit_val {
                    // TODO: Generic values
                    WfExitValue::Normal(result) => {
                        workflow_command::Variant::CompleteWorkflowExecution(
                            CompleteWorkflowExecution {
                                result: Some(result),
                            },
                        )
                    }
                    WfExitValue::ContinueAsNew(cmd) => {
                        workflow_command::Variant::ContinueAsNewWorkflowExecution(*cmd)
                    }
                    WfExitValue::Cancelled => workflow_command::Variant::CancelWorkflowExecution(
                        CancelWorkflowExecution {},
                    ),
                    WfExitValue::Evicted => {
                        panic!("Don't explicitly return this")
                    }
                },
                Err(e) => workflow_command::Variant::FailWorkflowExecution(FailWorkflowExecution {
                    failure: Some(Failure {
                        message: e.to_string(),
                        ..Default::default()
                    }),
                }),
            };
            activation_cmds.push(cmd.into())
        }
        Ok(false)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum CommandID {
    Timer(u32),
    Activity(u32),
    ChildWorkflowStart(u32),
    ChildWorkflowComplete(u32),
    SignalExternal(u32),
    CancelExternal(u32),
    NexusOpStart(u32),
    NexusOpComplete(u32),
}

fn update_response(
    instance_id: String,
    resp: update_response::Response,
) -> workflow_command::Variant {
    UpdateResponse {
        protocol_instance_id: instance_id,
        response: Some(resp),
    }
    .into()
}
```
