//! Default [`TaskSpawner`](twatch_core::domain::common::spawner::TaskSpawner)
//! implementation backed by a tokio runtime or runtime handle.

use std::future::Future;
use std::pin::Pin;

use shaku::{Component, Module, ModuleBuildContext};
use twatch_core::domain::common::spawner::TaskSpawner;

/// Spawns background futures onto either an owned runtime (the TUI) or the
/// current runtime handle (the daemon, which already runs inside tokio).
pub struct TokioTaskSpawner {
    pub rt: Option<tokio::runtime::Runtime>,
    pub handle: Option<tokio::runtime::Handle>,
}

/// Shaku parameters for [`TokioTaskSpawner`].
#[derive(Default)]
pub struct TokioTaskSpawnerParameters {
    pub rt: Option<tokio::runtime::Runtime>,
    pub handle: Option<tokio::runtime::Handle>,
}

impl<M: Module> Component<M> for TokioTaskSpawner {
    type Interface = dyn TaskSpawner;
    type Parameters = TokioTaskSpawnerParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        assert!(
            params.rt.is_some() || params.handle.is_some(),
            "TokioTaskSpawner: rt or handle parameter required"
        );
        Box::new(Self { rt: params.rt, handle: params.handle })
    }
}

impl TaskSpawner for TokioTaskSpawner {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        if let Some(runtime) = &self.rt {
            runtime.spawn(future);
        } else if let Some(handle) = &self.handle {
            handle.spawn(future);
        }
    }
}
