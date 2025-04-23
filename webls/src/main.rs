#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::path::PathBuf;
    let mut args = std::env::args();
    args.next();

    let root = args.next().and_then(|x| x.parse::<PathBuf>().ok());
    let port = args.next().and_then(|x| x.parse::<u16>().ok());
    webls::serve(root, port).await;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
