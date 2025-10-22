use std::{
    env::args,
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub(crate) fn path_as_url(path: &Path) -> String {
    path.iter()
        .fold(String::new(), |acc, x| acc + "/" + x.to_str().unwrap())
}

pub fn app_name_url() -> String {
    args()
        .next()
        .and_then(|x| x.parse::<PathBuf>().ok())
        .and_then(|x| {
            x.file_name()
                .and_then(|x| x.to_str().map(|x| x.to_string()))
        })
        .map(|x| format!("/{}", x))
        .unwrap()
}

pub(crate) fn path_as_query(path: &Path) -> String {
    let mut it = path.iter();
    let kv = |(i, x): (_, &OsStr)| format!("{}={}", i, x.to_str().unwrap());

    let first = it
        .next()
        .map(|x| String::from("?") + &kv((0, x)))
        .unwrap_or_default();

    it.enumerate()
        .map(|(i, x)| (i + 1, x))
        .map(kv)
        .fold(first, |acc, x| acc + "&&" + &x)
}
