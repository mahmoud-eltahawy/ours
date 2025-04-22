use std::path::PathBuf;
use tokio::io;
use tokio::process::Command;

use serde::{Deserialize, Serialize};

pub async fn refresh_partitions(path: PathBuf) -> io::Result<()> {
    let lsblk = Lsblk::get().await?;
    for p in lsblk.partitions().iter() {
        let mut path = path.clone();
        path.push(&p.name);
        if p.is_mounted() {
            p.umount(path.clone()).await?;
        }
        p.mount(path).await?;
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Lsblk {
    blockdevices: Vec<BlockDevice>,
}

const MINIMUM_STORAGE_SIZE_IN_GIGABYTES: f32 = 7.;

impl Lsblk {
    async fn get() -> io::Result<Self> {
        let bytes = Command::new("lsblk").arg("--json").output().await?.stdout;
        let result = serde_json::from_slice::<Self>(&bytes).unwrap();
        Ok(result)
    }

    fn partitions(self) -> Vec<Partition> {
        let Self { blockdevices } = self;
        blockdevices
            .into_iter()
            .flat_map(|BlockDevice { children }| children)
            .filter(Partition::valid_size)
            .filter(Partition::is_system_free)
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockDevice {
    children: Vec<Partition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Partition {
    name: String,
    size: String,
    mountpoints: Vec<Option<PathBuf>>,
}

impl Partition {
    fn valid_size(&self) -> bool {
        let bigger_than = move |num: f32| {
            self.size[0..self.size.len() - 1]
                .parse::<f32>()
                .is_ok_and(|x| x > num)
        };
        if self.size.ends_with("T") && bigger_than(MINIMUM_STORAGE_SIZE_IN_GIGABYTES / 1000.) {
            return true;
        }
        self.size.ends_with("G") && bigger_than(MINIMUM_STORAGE_SIZE_IN_GIGABYTES)
    }
    fn is_system_free(&self) -> bool {
        !self.mountpoints.iter().any(|x| {
            x.as_ref()
                .is_some_and(|s| matches!(s.to_str().unwrap(), "/" | "/home" | "/boot"))
        })
    }

    fn dev_path(&self) -> PathBuf {
        let mut dev_path = PathBuf::new();
        dev_path.push("/dev");
        dev_path.push(self.name.clone());
        dev_path
    }

    fn is_mounted(&self) -> bool {
        self.mountpoints.iter().any(|x| x.as_ref().is_some())
    }

    async fn mount(&self, path: PathBuf) -> io::Result<()> {
        let _ = tokio::fs::create_dir(&path).await;
        let _ = Command::new("mount")
            .args([self.dev_path(), path])
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }

    async fn umount(&self, mut path: PathBuf) -> io::Result<()> {
        path.push(&self.name);
        let _ = Command::new("umount")
            .arg(self.dev_path())
            .spawn()?
            .wait()
            .await?;
        let _ = tokio::fs::remove_dir(path).await;
        Ok(())
    }
}
