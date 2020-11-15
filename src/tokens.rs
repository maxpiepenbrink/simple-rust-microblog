use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SiteContent {
    pub file_name: String,
    pub title: String,
    pub timestamp: u128,
    pub page_tokens: Vec<PageToken>,
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