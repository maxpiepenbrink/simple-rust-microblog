use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct SiteContent {
    pub file_name: String,
    pub title: String,
    pub timestamp: u128,
    pub page_tokens: Vec<PageToken>,
}

impl PartialEq for SiteContent {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.timestamp == other.timestamp
    }
}

impl Eq for SiteContent {}

impl PartialOrd for SiteContent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SiteContent {
    fn cmp(&self, other: &Self) -> Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Tag(Box<str>, String),
    Span(String),
    Newline,
    StartParagraph,
    EndParagraph,
    EOF,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PageToken {
    pub token_type: String,
    pub meta: HashMap<String, String>,
}