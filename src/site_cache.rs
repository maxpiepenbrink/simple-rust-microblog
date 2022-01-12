extern crate lazy_static;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::tokens::SiteContent;

// https://users.rust-lang.org/t/solved-what-pattern-would-you-suggest-for-caching-since-theres-no-concept-of-global-heap-variables-in-rust/26086
type Data = HashMap<String, SiteContent>;
type ImageData = HashMap<String, String>;
lazy_static! {
    pub static ref SITE_CONTENT_CACHE: RwLock<Data> = RwLock::new(HashMap::new());
    pub static ref SITE_LINK_CACHE: RwLock<ImageData> = RwLock::new(HashMap::new());
}

pub fn create_link(name: &String, path: &Path) {
    let mut cache = SITE_LINK_CACHE.write().unwrap();

    let http_name = name.clone();
    let os_path = String::from(path.to_str().unwrap());
    println!("Storing ({}) -> ({})", http_name, os_path);

    cache.insert(http_name, os_path);
}

pub fn get_page_root(page_id: &String) -> Option<String> {
    let cache = SITE_LINK_CACHE.read().unwrap();
    if let Some(page_path_str) = cache.get(page_id) {
        return Some(page_path_str.clone());
    }
    return None;
}

pub fn cache_page(name: String, content: SiteContent) {
    let mut cache = SITE_CONTENT_CACHE.write().unwrap();
    cache.insert(name, content);
}

pub fn get_all_site_content() -> Vec<SiteContent> {
    let cache = SITE_CONTENT_CACHE.read().unwrap();
    let mut ret = Vec::new();
    for (_, page) in cache.iter() {
        ret.push(page.clone());
    }
    return ret;
}