use std::path::PathBuf;
use tokio::io;
use tokio::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Lsblk {
    blockdevices: Vec<BlockDevice>,
}

const MINIMUM_STORAGE_SIZE_IN_GIGABYTES: f32 = 7.;

impl Lsblk {
    pub async fn get() -> Self {
        let bytes = Command::new("lsblk")
            .arg("--json")
            .output()
            .await
            .unwrap()
            .stdout;
        serde_json::from_slice::<Self>(&bytes).unwrap()
    }

    pub fn partitions(self) -> Vec<Partition> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Partition {
    name: String,
    size: String,
    mountpoints: Vec<Option<String>>,
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
                .is_some_and(|s| matches!(s.as_str(), "/" | "/home" | "/boot"))
        })
    }

    pub async fn mount(&self, path: PathBuf) -> io::Result<()> {
        let mut dev_path = PathBuf::new();
        dev_path.push("/dev");
        dev_path.push(self.name.clone());
        let _ = Command::new("mount")
            .args([dev_path, path])
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }

    pub async fn umount(&self) -> io::Result<()> {
        let mut dev_path = PathBuf::new();
        dev_path.push("/dev");
        dev_path.push(self.name.clone());
        let _ = Command::new("umount").arg(dev_path).spawn()?.wait().await?;
        Ok(())
    }
}
