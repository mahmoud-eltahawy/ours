use crate::app::nav_bar::Tool;
use leptos::{ev, html::Ul, prelude::*};
use leptos_use::{on_click_outside, use_event_listener, use_window};
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use std::path::Path;

#[cfg(feature = "ssr")]
impl From<&sysinfo::Disk> for Disk {
    fn from(value: &sysinfo::Disk) -> Self {
        Self {
            total_space: value.total_space(),
            available_space: value.available_space(),
            name: value
                .mount_point()
                .file_name()
                .map(|x| x.to_str().unwrap().to_string())
                .unwrap_or(String::from("root")),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Disk {
    total_space: u64,
    available_space: u64,
    name: String,
}

#[component]
pub fn Info() -> impl IntoView {
    let display = RwSignal::new(false);

    let onclick = move || {
        display.set(true);
    };

    let active = move || true;
    view! {
       <Tool name="info" active onclick />
       <Show when=move || display.get()>
           <InfoCard display/>
       </Show>
    }
}

use server_fn::codec::Cbor;
#[server(
    input = Cbor,
    output = Cbor
)]
async fn get_disks() -> Result<Vec<Disk>, ServerFnError> {
    let boot = Path::new("/boot");
    let disks = sysinfo::Disks::new_with_refreshed_list()
        .list()
        .into_iter()
        .filter(|x| x.mount_point() != boot)
        .map(Disk::from)
        .collect();
    Ok(disks)
}

#[component]
fn InfoCard(display: RwSignal<bool>) -> impl IntoView {
    let disks = Resource::new(|| (), move |_| get_disks());

    Effect::new(move || {
        if display.get() {
            disks.refetch();
        }
    });

    let _ = use_event_listener(use_window(), ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            display.set(false);
        }
    });

    let target = NodeRef::<Ul>::new();

    let _ = on_click_outside(target, move |_| {
        display.set(false);
    });

    view! {
        <Suspense>
            <ul
                class="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2"
                node_ref={target}
            >
                <For
                    each=move ||disks.get().transpose().ok().flatten().unwrap_or(Vec::new())
                    key=|x| x.name.clone()
                    let:disk
                >
                    <DiskInfo disk/>
                </For>
            </ul>
        </Suspense>
    }
}

#[component]
fn DiskInfo(disk: Disk) -> impl IntoView {
    let used_space = disk.total_space - disk.available_space;
    let free = format!(
        "FREE : {:.2}G",
        disk.available_space as f64 / 1024.0f64.powi(3)
    );
    let used = format!("USED : {:.2}G", used_space as f64 / 1024.0f64.powi(3));
    let usage = format!(
        "{:.2}%",
        (used_space as f64 / disk.total_space as f64) * 100.
    );
    view! {
        <li>
            <h3 class="text-3xl m-5">{disk.name}</h3>
            <div class="grid grid-cols-2 gap-5">
                <progress value={used_space.to_string()} max={disk.total_space.to_string()}/>
                <span>{usage}</span>
                <div class="grid grid-cols-2 gap-5">
                    <span>{used}</span>
                    <span>{free}</span>
                </div>
            </div>
        </li>
    }
}
