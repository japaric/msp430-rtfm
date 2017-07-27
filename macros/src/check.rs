use std::collections::HashMap;

use syntax::error::*;
use syntax::{self, Idents, Statics};
use syntax::check::{self, Idle, Init};
use syn::{Ident, Path};

pub type Tasks = HashMap<Ident, Task>;

pub struct App {
    pub device: Path,
    pub idle: Idle,
    pub init: Init,
    pub resources: Statics,
    pub tasks: Tasks,
}

pub struct Task {
    pub path: Option<Path>,
    pub resources: Idents,
}

pub fn app(app: check::App) -> Result<App> {
    let app = App {
        device: app.device,
        idle: app.idle,
        init: app.init,
        resources: app.resources,
        tasks: app.tasks
            .into_iter()
            .map(|(k, v)| {
                let name = k.clone();
                Ok((
                    k,
                    ::check::task(v)
                        .chain_err(|| format!("checking task `{}`", name))?,
                ))
            })
            .collect::<Result<_>>()
            .chain_err(|| "checking `tasks`")?,
    };

    ::check::resources(&app)
        .chain_err(|| "checking `resources`")?;

    Ok(app)
}

fn resources(app: &App) -> Result<()> {
    for resource in app.resources.keys() {
        if app.idle.resources.contains(resource) {
            continue;
        }

        if app.tasks
            .values()
            .any(|task| task.resources.contains(resource))
        {
            continue;
        }

        bail!("resource `{}` is unused", resource);
    }

    Ok(())
}

fn task(task: syntax::check::Task) -> Result<Task> {
    ensure!(
        task.enabled.is_none(),
        "should not contain an `enabled` field"
    );

    ensure!(
        task.priority.is_none(),
        "should not contain a `priority` field"
    );

    Ok(Task {
        path: task.path,
        resources: task.resources,
    })
}
