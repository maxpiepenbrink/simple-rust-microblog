#![feature(proc_macro_hygiene, decl_macro, map_into_keys_values)]

#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::{io, thread};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::tokens::PageToken;
use rocket::response::NamedFile;
use rocket::Config;
use rocket::config::Environment;

mod content_compiler;
mod tokens;
mod site_cache;
mod content_monitor;

fn main() {
    content_compiler::load_site_content();

    // watch the content/ path for any changes
    let handle = thread::spawn(|| {
        content_monitor::start_monitor();
    });

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .port(8000)
        .finalize().unwrap();

    //rocket::ignite()
    rocket::custom(config)
        .attach(Template::fairing())
        .mount("/", routes![index, archive_list, site_content, get_static])
        .launch();
}

#[get("/site-content/<page_hash>/<file..>")]
fn site_content(page_hash: String, file: PathBuf) -> Option<NamedFile> {
    if let Some(root) = site_cache::get_page_root(&page_hash) {
        let root_path = Path::new(&root);
        return NamedFile::open(root_path.join(file)).ok()
    }
    return None
}

#[get("/static/<file..>")]
fn get_static(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

fn footer() -> Vec<PageToken> {
    let mut meta = HashMap::new();

    return vec![PageToken {
        token_type: String::from("footer"),
        meta,
    }];
}

#[derive(Serialize, Debug)]
struct RenderedPage {
    title: String,
    body: Vec<PageToken>,
}

#[get("/")]
fn index() -> Template {
    let mut pages = site_cache::get_all_site_content();
    pages.sort();

    let tokens: Vec<PageToken> = pages.into_iter().map(|p| { p.page_tokens })
        .intersperse(footer())
        .flat_map(|x| x)
        .collect();

    let context = RenderedPage {
        title: String::from("Max's Thoughts & Feelings"),
        body: tokens,
    };

    Template::render("index", &context)
}

#[get("/archive")]
fn archive_list() -> String {
    String::from("list all entries")
}

// TODO: archive view
// #[get("/archive/<path>")]
// fn archive_item(path: &RawStr) -> Page {
// }

