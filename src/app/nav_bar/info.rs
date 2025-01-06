use crate::app::nav_bar::Tool;
use leptos::{ev, prelude::*};
use leptos_use::{use_event_listener, use_window};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "ssr")]
use {
    crate::ServerContext,
    std::path::{Path, StripPrefixError},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MyDisks {
    root: Disk,
    others: Vec<Disk>,
}

#[cfg(feature = "ssr")]
impl MyDisks {
    fn new() -> Self {
        let disks = sysinfo::Disks::new_with_refreshed_list();
        let is_a = move |disk: &&sysinfo::Disk, a| disk.mount_point() == Path::new(a);
        let is_a_root = |x| is_a(x, "/");
        let list = disks
            .list()
            .iter()
            .filter(|x| !is_a(x, "/boot"))
            .collect::<Vec<_>>();
        let root = list.iter().find(|x| is_a_root(x)).unwrap();
        let root = Disk::from(root);
        let others = list
            .iter()
            .filter(|x| !is_a_root(x))
            .map(Disk::from)
            .collect();
        Self { root, others }
    }
    fn strip_prefix(&mut self, path: &PathBuf) -> Result<(), StripPrefixError> {
        self.root.strip_prefix(path)?;
        for disk in &mut self.others {
            disk.strip_prefix(path)?
        }
        Ok(())
    }
}
#[cfg(feature = "ssr")]
impl From<&&sysinfo::Disk> for Disk {
    fn from(value: &&sysinfo::Disk) -> Self {
        Self {
            total_space: value.total_space(),
            available_space: value.available_space(),
            path: value.mount_point().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Disk {
    total_space: u64,
    available_space: u64,
    path: PathBuf,
}

impl Disk {
    fn usage(&self) -> f64 {
        (self.available_space as f64 / self.total_space as f64) * 100.
    }

    #[cfg(feature = "ssr")]
    fn strip_prefix(&mut self, path: &PathBuf) -> Result<(), StripPrefixError> {
        self.path = self.path.strip_prefix(&path)?.into();
        Ok(())
    }
}

#[component]
pub fn Info() -> impl IntoView {
    let display = RwSignal::new(false);

    let onclick = move || {
        display.update(|x| *x = !*x);
    };

    let active = move || true;
    view! {
       <Tool name="info" active onclick />
       <Show when=move || display.get()>
           <InfoCard dis=display/>
       </Show>
    }
}

use server_fn::codec::Cbor;
#[server(
    input = Cbor,
    output = Cbor
)]
async fn get_disks() -> Result<MyDisks, ServerFnError> {
    let context = use_context::<ServerContext>().unwrap();
    let mut disks = MyDisks::new();
    let _ = disks.strip_prefix(&context.root);
    Ok(disks)
}

#[component]
fn InfoCard(dis: RwSignal<bool>) -> impl IntoView {
    let disks = Resource::new(|| (), move |_| get_disks());

    let display = move || {
        disks
            .get()
            .transpose()
            .ok()
            .flatten()
            .map(|x| format!("{:#?}", x))
    };

    Effect::new(move || {
        if dis.get() {
            disks.refetch();
        }
    });

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            dis.set(false);
        }
    });

    view! {
        <Suspense>
            <div class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <pre>{display}</pre>
            </div>
        </Suspense>
    }
}
