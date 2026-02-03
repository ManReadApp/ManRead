use std::{
    any::{Any, TypeId},
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use tokio::{spawn, sync::Notify, task::JoinHandle};
pub type GroupId = u64;
pub enum ProcessType {
    /// Needs to run when no other process is running
    Alone,
    /// Needs to run when no other process is running in Group & TypeId
    KindGroup(GroupId),
    /// Needs to run when no other process is running in Group
    Group(GroupId),
    /// Needs to run when no other process is running in TypeId
    Kind,
    /// Does not care about other processes
    Parallel,
}

#[derive(Default)]
pub struct RunningInfo {
    ids: Vec<TypeId>,
    groups: Vec<GroupId>,
}

impl RunningInfo {
    pub fn extend(&mut self, item: &dyn Event) {
        self.ids.push(item.id());
        if let Some(gid) = item.group_id() {
            self.groups.push(gid);
        }
    }
}

pub trait Event: Any + Send + Sync {
    fn id(&self) -> TypeId {
        self.type_id()
    }
    fn group_id(&self) -> Option<GroupId> {
        None
    }
    fn wait_before_run(&self) -> Duration {
        Duration::ZERO
    }
    fn rerun(&self) -> Option<Duration> {
        None
    }
    fn needs_wait(&self, running: &RunningInfo) -> bool {
        match self.parallel() {
            ProcessType::Alone => running.ids.len() != 0,
            ProcessType::Group(g) => running.groups.contains(&g),
            ProcessType::Kind => running.ids.contains(&self.id()),
            ProcessType::Parallel => false,
            ProcessType::KindGroup(g) => {
                running.groups.contains(&g) || running.ids.contains(&self.id())
            }
        }
    }

    fn spawn_execute(&self, notify: Arc<Notify>) {
        let f = self.execute();
        let handle = spawn(async move {
            f.await;
            notify.notify_one();
        });
        self.set_handle(handle);
    }

    fn execute(&self) -> Pin<Box<dyn Future<Output = ()> + Send>>;
    fn cancel(&self);
    fn is_running(&self) -> bool;
    fn parallel(&self) -> ProcessType;
    fn set_handle(&self, handle: JoinHandle<()>);
}
