use axum::Json;
use serde::Serialize;

use crate::ServerResult;

#[derive(Serialize)]
pub struct Disk {
    pub total_space: u64,
    pub available_space: u64,
    pub name: String,
}

pub async fn get_disks() -> ServerResult<Json<Vec<Disk>>> {
    use std::path::Path;
    fn from(value: &sysinfo::Disk) -> Disk {
        Disk {
            total_space: value.total_space(),
            available_space: value.available_space(),
            name: value
                .mount_point()
                .file_name()
                .map(|x| x.to_str().unwrap().to_string())
                .unwrap_or(String::from("root")),
        }
    }
    let boot = Path::new("/boot");
    let disks = sysinfo::Disks::new_with_refreshed_list()
        .list()
        .iter()
        .filter(|x| x.mount_point() != boot)
        .map(from)
        .collect();
    Ok(Json(disks))
}
