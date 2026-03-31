//! mdBook preprocessor that adds subsection numbering to chapter headings.

use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use std::io;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("supports") => {
            // Supports all renderers.
            return;
        }
        Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        }
        None => {}
    }

    if let Err(e) = handle_preprocessing() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

struct SubsectionNumbering;

impl Preprocessor for SubsectionNumbering {
    fn name(&self) -> &str {
        "subsection-numbering"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let mut chapter_num = 0;

        if !try_add_cover_to_intro(&mut book.items) {
            let _ = add_cover_to_first_chapter(&mut book.items);
        }

        // Process each chapter with its number
        process_book_items(&mut book.items, &mut chapter_num);

        Ok(book)
    }
}

fn process_book_items(items: &mut Vec<BookItem>, chapter_num: &mut usize) {
    for item in items {
        if let BookItem::Chapter(ref mut chapter) = item {
            // Skip unnumbered chapters (like Introduction)
            if chapter.number.is_some() {
                *chapter_num += 1;
                add_subsection_numbers(chapter, *chapter_num);
            }

            // Process nested chapters recursively
            process_book_items(&mut chapter.sub_items, chapter_num);
        }
    }
}

fn try_add_cover_to_intro(items: &mut Vec<BookItem>) -> bool {
    for item in items {
        if let BookItem::Chapter(chapter) = item {
            if is_intro_chapter(chapter) {
                add_print_cover_before_intro(chapter);
                return true;
            }

            if try_add_cover_to_intro(&mut chapter.sub_items) {
                return true;
            }
        }
    }

    false
}

fn add_cover_to_first_chapter(items: &mut Vec<BookItem>) -> bool {
    for item in items {
        if let BookItem::Chapter(chapter) = item {
            add_print_cover_before_intro(chapter);
            return true;
        }
    }

    false
}

fn is_intro_chapter(chapter: &Chapter) -> bool {
    if let Some(path) = &chapter.path {
        if path.to_string_lossy().ends_with("intro.md") {
            return true;
        }
    }

    let normalized_name = chapter
        .name
        .chars()
        .filter(|c| !c.is_ascii_digit() && *c != '.' && !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    if normalized_name == "introduction" {
        return true;
    }

    chapter
        .content
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("# "))
        .map(|line| {
            line.trim_start_matches("# ")
                .chars()
                .filter(|c| !c.is_ascii_digit() && *c != '.' && !c.is_whitespace())
                .collect::<String>()
                .eq_ignore_ascii_case("introduction")
        })
        .unwrap_or(false)
}

fn add_print_cover_before_intro(chapter: &mut Chapter) {
    if chapter.content.contains("print-cover-page") {
        return;
    }

    let cover = r#"<div class="print-cover-page">
  <img src="images/kubook.png" alt="Kubook cover" />
</div>"#;

    chapter.content = format!("{cover}\n\n{}", chapter.content);
}

fn add_subsection_numbers(chapter: &mut Chapter, chapter_num: usize) {
    let lines: Vec<&str> = chapter.content.lines().collect();
    let mut result = String::new();
    let mut h2_num = 0;
    let mut h3_num = 0;
    let mut h4_num = 0;

    for line in lines {
        let trimmed = line.trim_start();

        // Check for H1 headers (#)
        if trimmed.starts_with("# ") && !trimmed.starts_with("## ") {
            let heading_text = trimmed.trim_start_matches("# ").trim();
            // Check if already numbered
            if !heading_text.chars().next().map_or(false, |c| c.is_digit(10)) {
                let indent = &line[..line.len() - trimmed.len()];
                result.push_str(&format!("{}# {} {}\n", indent, chapter_num, heading_text));
                continue;
            }
        }
        // Check for H2 headers (##)
        else if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            h2_num += 1;
            h3_num = 0; // Reset H3 counter
            h4_num = 0; // Reset H4 counter
            let heading_text = trimmed.trim_start_matches("## ").trim();
            // Check if already numbered
            if !heading_text.chars().next().map_or(false, |c| c.is_digit(10)) {
                let indent = &line[..line.len() - trimmed.len()];
                result.push_str(&format!("{}## {}.{} {}\n", indent, chapter_num, h2_num, heading_text));
                continue;
            }
        }
        // Check for H3 headers (###)
        else if trimmed.starts_with("### ") && !trimmed.starts_with("#### ") {
            h3_num += 1;
            h4_num = 0; // Reset H4 counter
            let heading_text = trimmed.trim_start_matches("### ").trim();
            // Check if already numbered
            if !heading_text.chars().next().map_or(false, |c| c.is_digit(10)) {
                let indent = &line[..line.len() - trimmed.len()];
                result.push_str(&format!("{}### {}.{}.{} {}\n", indent, chapter_num, h2_num, h3_num, heading_text));
                continue;
            }
        }
        // Check for H4 headers (####)
        else if trimmed.starts_with("#### ") && !trimmed.starts_with("##### ") {
            h4_num += 1;
            let heading_text = trimmed.trim_start_matches("#### ").trim();
            // Check if already numbered
            if !heading_text.chars().next().map_or(false, |c| c.is_digit(10)) {
                let indent = &line[..line.len() - trimmed.len()];
                result.push_str(&format!("{}#### {}.{}.{}.{} {}\n", indent, chapter_num, h2_num, h3_num, h4_num, heading_text));
                continue;
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    chapter.content = result;
}

pub fn handle_preprocessing() -> Result<()> {
    let pre = SubsectionNumbering;
    let (ctx, book) = mdbook_preprocessor::parse_input(io::stdin())?;

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}
