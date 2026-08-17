//! Default [`TaskSpawner`](crate::domain::common::spawner::TaskSpawner)
//! implementation backed by a tokio runtime.

use crate::domain::common::spawner::TaskSpawner;
use shaku::{Component, Module, ModuleBuildContext};
use std::future::Future;
use std::pin::Pin;

/// Spawns background futures onto a user-provided tokio runtime.
pub struct TokioTaskSpawner {
    pub rt: tokio::runtime::Runtime,
}

/// Shaku parameters for [`TokioTaskSpawner`].
#[derive(Default)]
pub struct TokioTaskSpawnerParameters {
    pub rt: Option<tokio::runtime::Runtime>,
}

impl<M: Module> Component<M> for TokioTaskSpawner {
    type Interface = dyn TaskSpawner;
    type Parameters = TokioTaskSpawnerParameters;

    #[allow(clippy::expect_used)]
    fn build(
        _context: &mut ModuleBuildContext<M>,
        params: Self::Parameters,
    ) -> Box<Self::Interface> {
        Box::new(Self { rt: params.rt.expect("TokioTaskSpawner: rt parameter required") })
    }
}

impl TaskSpawner for TokioTaskSpawner {
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.rt.spawn(future);
    }
}
