// #[derive(Debug, Default)]
// pub struct Entry {
//     pub alias: Option<String>,
//     pub path: String,
// }

// impl From<&str> for Entry {
//     fn from(path: &str) -> Self {
//         Self {
//             alias: None,
//             path: path.to_string().into(),
//         }
//     }
// }


#[derive(Debug)]
pub struct BundleOptions {
    pub entries: Vec<String>,
}
