use std::collections::HashMap;

use quote::Tokens;
use rtfm_syntax::error::*;
use rtfm_syntax::{Idle, Idents, Init, Statics};
use syn::Ident;

pub type Tasks = HashMap<Ident, Task>;

pub struct App {
    pub device: Tokens,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

pub struct Task {
    pub resources: Idents,
}

pub fn app(app: ::rtfm_syntax::App) -> Result<App> {
    let mut tasks = HashMap::new();

    for (k, v) in app.tasks {
        tasks.insert(k, ::check::task(v)?);
    }

    for resource in app.resources.keys() {
        if app.idle.resources.contains(resource) {
            continue;
        }

        if tasks.values().any(|task| task.resources.contains(resource)) {
            continue;
        }

        bail!("resource `{}` is unused", resource);
    }

    Ok(App {
        device: app.device,
        idle: app.idle,
        init: app.init,
        resources: app.resources,
        tasks: tasks,
    })
}

fn task(task: ::rtfm_syntax::Task) -> Result<Task> {
    ensure!(
        task.enabled.is_none(),
        "should not contain an `enabled` field"
    );

    ensure!(
        task.priority.is_none(),
        "should not contain a `priority` field"
    );

    Ok(Task {
        resources: task.resources,
    })
}
