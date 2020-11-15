extern crate lazy_static;

use std::collections::HashMap;
use std::sync::RwLock;

use lazy_static::lazy_static;

use crate::tokens::SiteContent;

// https://users.rust-lang.org/t/solved-what-pattern-would-you-suggest-for-caching-since-theres-no-concept-of-global-heap-variables-in-rust/26086
type Data = HashMap<String, SiteContent>;
lazy_static! {
    pub static ref SITE_CONTENT_CACHE: RwLock<Data> = RwLock::new(HashMap::new());
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