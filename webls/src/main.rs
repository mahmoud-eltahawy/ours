#[cfg(feature = "ssr")]
use std::path::PathBuf;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();

    let root = args.next().map(|x| x.parse::<PathBuf>().unwrap());
    let port = args.next().map(|x| x.parse::<u16>().unwrap());
    webls::serve(root, port).await;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
