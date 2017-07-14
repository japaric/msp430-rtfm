use std::collections::HashMap;

use syn::Ident;

use check::App;

pub type Ownerships = HashMap<Ident, Ownership>;

#[derive(Clone, Copy, PartialEq)]
pub enum Ownership {
    /// Co-owned by tasks that run at the same priority
    Owned(Owner),
    /// Shared by tasks that run at different priorities
    // The only possible owners here are idle and at least one interrupt
    Shared,
}

impl Ownership {
    pub fn is_owned(&self) -> bool {
        *self != Ownership::Shared
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Owner {
    Idle,
    Interrupt,
}

pub fn app(app: &App) -> Ownerships {
    let mut ownerships = HashMap::new();

    for resource in &app.idle.resources {
        ownerships.insert(resource.clone(), Ownership::Owned(Owner::Idle));
    }

    for task in app.tasks.values() {
        for resource in &task.resources {
            if let Some(ownership) = ownerships.get_mut(resource) {
                if *ownership == Ownership::Owned(Owner::Idle) {
                    *ownership = Ownership::Shared
                }

                continue;
            }

            ownerships
                .insert(resource.clone(), Ownership::Owned(Owner::Interrupt));
        }
    }

    ownerships
}
