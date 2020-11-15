#![feature(proc_macro_hygiene, decl_macro, map_into_keys_values)]

#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};

use crate::tokens::PageToken;

mod content_compiler;
mod tokens;
mod site_cache;

fn main() {
    content_compiler::load_site_content();

    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index, archive_list])
        .launch();
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
    let pages = site_cache::get_all_site_content();

    let tokens: Vec<PageToken> = pages.into_iter().map(|p| {
        p.page_tokens
    }).intersperse(footer()).flat_map(|x| x).collect();

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
//
// #[get("/archive/<path>")]
// fn archive_item(path: &RawStr) -> Page {
// }

