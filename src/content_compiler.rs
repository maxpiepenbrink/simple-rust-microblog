use std::{fs, io};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{DirEntry, Metadata};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::CharIndices;
use std::time::{SystemTime, UNIX_EPOCH};

use rocket::http::ext::IntoCollection;
use walkdir::WalkDir;

use crate::site_cache;
use crate::tokens::*;
use crate::tokens::Token::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redundant_lines_are_removed() {
        let tokens = remove_redundant_newlines(vec![
            Newline,
            Newline,
            Newline,
            EOF,
        ]);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn single_newline_is_removed() {
        let tokens: Vec<_> = remove_redundant_newlines(vec![
            Newline,
            Span("ignored".to_string()),
            Newline,
            Span("ignored".to_string()),
            Newline,
            EOF,
        ])
            .into_iter()
            .filter(|x| *x == Newline)
            .collect();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn contiguous_spans_get_merged() {
        let tokens: Vec<_> = merge_spans(vec![
            Span("Greetings to".into()),
            Span("the".into()),
            Span("world!".into()),
            EOF
        ]);
        println!("tokens: {:?}", tokens);
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn paragraph_creation() {
        let tokens: Vec<_> = create_paragraphs(vec![
            Tag("header".into(), "ignored".into()),
            Span("This is a paragraph!".into()),
            Newline,
            Span("This is another paragraph".into()),
            Tag("url".into(), "ignored".into()),
            Span("This is another paragraph".into()),
            EOF,
        ]);

        assert!(matches!(tokens[0], Tag{..}));
        assert!(matches!(tokens[1], StartParagraph));
        assert!(matches!(tokens[2], Span(_)));
        assert!(matches!(tokens[3], EndParagraph));
        assert!(matches!(tokens[4], StartParagraph));
        assert!(matches!(tokens[5], Span(_)));
        assert!(matches!(tokens[6], Tag{..}));
        assert!(matches!(tokens[7], Span(_)));
        assert!(matches!(tokens[8], EndParagraph));
    }

    #[test]
    fn test_url_mapping() {
        let page_tokens = vec![
            PageToken {
                token_type: "image".into(),
                meta: HashMap::from_iter(
                    vec![
                        ("image".to_string(), "image.png".to_string())
                    ].into_iter()),
            }
        ];

        let page_tokens = create_file_links(page_tokens, Path::new("test"), &"hash".to_string());

        assert_eq!("site-content/hash/image.png", page_tokens.get(0).unwrap().meta.get("image").unwrap())
    }
}

const NON_PARAGRAPH_TAG_TYPES: &'static [&'static str] = &[
    &"header",
    &"title",
];

pub fn load_site_content() {
    let entries = WalkDir::new("./content/")
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|e| e.is_file())
        .filter(|e| e.extension().unwrap() == "hmm")
        .collect::<Vec<PathBuf>>();

    let content_files = entries.iter()
        .filter_map(|x| x.to_str())
        .collect::<Vec<_>>();
    println!("Processing content files: {}", content_files.join(", "));

    let results = entries.into_iter()
        .map(|file| compile_content(file))
        .collect::<Vec<_>>();

    println!("{:?}", results);
    println!("Loading pages into global cache...");
    for page in results {
        if page.is_err() { continue; }
        let p = page.unwrap();
        site_cache::cache_page(p.file_name.clone(), p);
    }
}

fn create_paragraphs(tokens: Vec<Token>) -> Vec<Token> {
    if !tokens.has_eof_token() { panic!("Invalid yo"); }

    let mut iter = tokens.into_iter();
    let mut new_tokens: Vec<Token> = Vec::new();

    let mut is_paragraph = false;
    while let Some(token) = iter.next() {
        match token {
            Span(_) => {
                if !is_paragraph {
                    new_tokens.push(StartParagraph);
                    is_paragraph = true;
                }
                new_tokens.push(token.clone());
            }
            Tag(ref tag_type, _) => {
                if is_paragraph && NON_PARAGRAPH_TAG_TYPES.contains(&tag_type.as_ref()) {
                    // we've noticed a paragraph and now there's a tag type which should end it
                    is_paragraph = false;
                    new_tokens.push(EndParagraph);
                } else if !is_paragraph && !NON_PARAGRAPH_TAG_TYPES.contains(&tag_type.as_ref()) {
                    new_tokens.push(StartParagraph);
                    is_paragraph = true;
                }
                new_tokens.push(token.clone());
            }
            Newline => {
                if is_paragraph {
                    new_tokens.push(EndParagraph);
                    is_paragraph = false;
                }
                // discard newlines
            }
            EOF => {
                if is_paragraph {
                    new_tokens.push(EndParagraph);
                    is_paragraph = false;
                }
            }
            _ => {
                new_tokens.push(token.clone());
            }
        }
    }

    return new_tokens;
}

// this just asserts w/e rules we need to know if a token is valid or not
trait TokenStreamValidator {
    fn has_eof_token(&self) -> bool;
}

impl TokenStreamValidator for Vec<Token> {
    fn has_eof_token(&self) -> bool {
        if let Some(EOF) = self.iter().rev().next() {
            return true;
        }
        return false;
    }
}

fn merge_spans(tokens: Vec<Token>) -> Vec<Token> {
    if !tokens.has_eof_token() { panic!("token stream ain't valid"); }

    let mut iter = tokens.into_iter();

    let mut new_tokens: Vec<Token> = Vec::new();

    let mut combined_spans: Option<Token> = Option::None;

    while let Some(token) = iter.next() {
        match token {
            Span(current_span_text) => {
                match combined_spans {
                    Some(Span(prev_text)) => {
                        combined_spans = Some(Span(format!("{} {}", prev_text, current_span_text)));
                    }
                    Some(_) => { panic!("This shouldn't be possible.") }
                    None => {
                        combined_spans = Some(Span(current_span_text));
                    }
                }
            }
            _ => {
                // this token is not a Span so let's close out the current span and add it to
                // our token vector, or just add w/e the current token is
                if combined_spans.is_some() {
                    new_tokens.push(combined_spans.unwrap());
                    combined_spans = None;
                }
                new_tokens.push(token);
            }
        }
    }

    return new_tokens;
}

fn compile_content(file: PathBuf) -> Result<SiteContent, &'static str> {

    // make sure it's a file
    let content_meta = fs::metadata(file.clone())
        .expect("Something went wrong reading the file");

    // open palm slam that shit into ram
    let contents = fs::read_to_string(file.clone())
        .expect("Something went wrong reading the file");

    // lex it
    let tokens = lex_content(contents);

    // do a few passes on the data to massage it into the right shape and generate new tokens,
    // remove redundant ones, etc.
    let tokens = remove_redundant_newlines(tokens);
    let tokens = merge_spans(tokens);
    let tokens = create_paragraphs(tokens);

    let page_tokens = convert_to_page_tokens(tokens);

    let local_page_path = file.parent().unwrap();

    let file_name: String = file.to_str().unwrap().into();
    let mut s = DefaultHasher::new();
    file_name.hash(&mut s);
    let title_hash = s.finish().to_string();
    let page_tokens = create_file_links(page_tokens, local_page_path, &title_hash);

    site_cache::create_link(&title_hash, local_page_path);

    let result = SiteContent {
        timestamp: decide_timestamp(&page_tokens, &content_meta),
        file_name: file_name.clone(),
        title: file_name,
        page_tokens,
    };
    return Ok(result);
}

fn create_file_links(tokens: Vec<PageToken>, root: &Path, title_hash: &String) -> Vec<PageToken> {
    let mut new_tokens: Vec<PageToken> = Vec::new();

    let mut iter = tokens.into_iter();
    while let Some(token) = iter.next() {
        match token {
            PageToken { token_type, mut meta } if token_type == "image" => {
                if let Some(url_path) = meta.get("image") {
                    // leave fully qualified uris alone
                    if !url_path.contains("://") {
                        let mut os_path = root.to_path_buf().clone();
                        os_path.push(url_path);

                        let site_path = format!("site-content/{}/{}", title_hash, url_path);

                        meta.insert("image".into(), site_path);
                    }
                }
                new_tokens.push(PageToken { token_type, meta });
            }
            _ => { new_tokens.push(token); }
        }
    }

    return new_tokens;
}

fn convert_to_page_tokens(tokens: Vec<Token>) -> Vec<PageToken> {
    let mut page_tokens: Vec<PageToken> = Vec::new();
    let mut token_iter = tokens.into_iter().peekable();
    loop {
        let token = token_iter.next();
        if let Some(token) = token {
            match token {
                Tag(tag_type, tag_string) => {
                    let splits: Vec<&str> = tag_string.split('|').collect();
                    let final_tags: Vec<(String, String)> = splits.into_iter().map(|tg| {
                        let components: Vec<&str> = tg.split(':').collect();
                        if components.len() > 2 { panic!("Too many ':'s in tag component"); }

                        let key = components.get(0).unwrap().to_string();
                        let val = components.get(1).unwrap_or(&"<empty>").to_string();
                        (key, val)
                    }).collect();

                    page_tokens.push(PageToken {
                        token_type: tag_type.into(),
                        meta: final_tags.into_iter().collect(),
                    })
                }
                Span(text) => {
                    page_tokens.push(PageToken {
                        token_type: "span".to_string(),
                        meta: vec![("text".to_string(), text.clone())].into_iter().collect(),
                    })
                }
                StartParagraph => {
                    page_tokens.push(PageToken {
                        token_type: "para_start".to_string(),
                        meta: HashMap::new(),
                    })
                }
                EndParagraph => {
                    page_tokens.push(PageToken {
                        token_type: "para_end".to_string(),
                        meta: HashMap::new(),
                    })
                }
                _ => {}
            }
        } else {
            break;
        }
    }
    return page_tokens;
}

fn decide_timestamp(tokens: &Vec<PageToken>, content_meta: &Metadata) -> u128 {
    for x in tokens {
        if x.token_type.eq("title") {
            if let Some(ts) = x.meta.get("timestamp") {
                return ts.parse::<u128>().unwrap();
            }
        }
    }

    if let Ok(ts) = content_meta.created() {
        if let Ok(ts) = ts.duration_since(UNIX_EPOCH) {
            return ts.as_millis();
        }
    }

    panic!("couldn't figure out a good timestamp for this post")
}

fn remove_redundant_newlines(tokens: Vec<Token>) -> Vec<Token> {
    let mut iter = tokens.into_iter().peekable();
    let mut new_tokens: Vec<Token> = Vec::new();

    let mut newline = false;

    while let Some(token) = iter.next() {
        match token {
            Newline => {
                if newline { continue; }
                newline = true;
                let next_token = iter.peek();
                if let Some(Newline) = next_token {
                    new_tokens.push(Newline);
                }
            }
            _ => {
                newline = false;
                new_tokens.push(token);
            }
        }
    }

    return new_tokens;
}

fn finish_span(span: &mut String, tokens: &mut Vec<Token>) {
    if span.trim().len() > 0 {
        // finish current span
        tokens.push(Span(span.clone()));
        span.clear();
    }
}

fn lex_tag(span: &mut String, char_iter: &mut CharIndices, tokens: &mut Vec<Token>) {
    let (_, next_ch) = char_iter.next().expect("Unexpected EOF");
    if next_ch == '[' {
        finish_span(span, tokens);

        let mut value = String::new();
        loop {
            let (_, next_ch) = char_iter.next().expect("Unexpected EOF");
            if next_ch == ']' { // stop building a tag
                break;
            }
            value.push(next_ch);
        }

        // grab the first token to guess at the type
        let s: Vec<&str> = value.split('|').collect();
        if s.len() > 0 {
            let i: Vec<&str> = s[0].split(':').collect();
            let tag_val = if i.len() > 0 { i[0] } else { s[0] };
            tokens.push(Tag(tag_val.into(), value));
        } else {
            tokens.push(Tag(value.clone().as_str().into(), value));
        }
    }
}

fn lex_content(contents: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut char_iter = contents.char_indices();
    let mut current_span = String::new();

    loop {
        // seek a token we know what to do with
        match char_iter.next() {
            Some((_, ch)) => {
                // todo: use match
                if ch == '\n' {       // -- note a newline
                    finish_span(&mut current_span, &mut tokens);
                    tokens.push(Newline);
                } else if ch == '#' { // -- note a tag
                    lex_tag(&mut current_span, &mut char_iter, &mut tokens);
                } else {
                    if ch != '\n' && ch != '\r' {
                        current_span.push(ch);
                    }
                }
            }
            None => { // reached EOF
                if current_span.trim().len() > 0 {
                    finish_span(&mut current_span, &mut tokens);
                }

                break;
            }
        }
    };

    tokens.push(EOF);

    //println!("{:?}", tokens);

    return tokens;
}