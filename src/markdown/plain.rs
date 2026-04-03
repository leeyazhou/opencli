use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub fn render_markdown(input: &str) -> String {
    let mut out = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut pending_link: Option<String> = None;

    for event in Parser::new_ext(input, Options::all()) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    if !out.ends_with('\n') && !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(heading_prefix(level));
                }
                Tag::BlockQuote(_) => {
                    if !out.ends_with('\n') && !out.is_empty() {
                        out.push('\n');
                    }
                    in_blockquote = true;
                }
                Tag::List(start) => list_stack.push(start),
                Tag::Item => {
                    if !out.ends_with('\n') && !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(&"  ".repeat(list_stack.len().saturating_sub(1)));
                    match list_stack.last_mut() {
                        Some(Some(n)) => {
                            out.push_str(&format!("{}. ", *n));
                            *n += 1;
                        }
                        _ => out.push_str("- "),
                    }
                }
                Tag::CodeBlock(kind) => {
                    if !out.ends_with('\n') && !out.is_empty() {
                        out.push('\n');
                    }
                    in_code_block = true;
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.into_string()),
                        _ => None,
                    };
                    out.push_str("```\n");
                }
                Tag::Link { dest_url, .. } => pending_link = Some(dest_url.to_string()),
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph | TagEnd::Heading(..) => push_blank_line(&mut out),
                TagEnd::BlockQuote(_) => {
                    in_blockquote = false;
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                TagEnd::List(_) | TagEnd::Item => {
                    if matches!(tag, TagEnd::List(_)) {
                        list_stack.pop();
                    }
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                TagEnd::CodeBlock => {
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                    out.push_str("```\n");
                    in_code_block = false;
                    code_lang = None;
                }
                TagEnd::Link => {
                    if let Some(url) = pending_link.take() {
                        out.push_str(&format!(" ({url})"));
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_blockquote && line_start(&out) {
                    out.push_str("> ");
                }
                if in_code_block && let Some(lang) = code_lang.take() {
                    out.push_str(&format!("# language: {lang}\n"));
                }
                out.push_str(&text);
            }
            Event::Code(text) => {
                out.push('`');
                out.push_str(&text);
                out.push('`');
            }
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::Rule => out.push_str("\n---\n"),
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::FootnoteReference(name) => out.push_str(&format!("[{name}]")),
            Event::TaskListMarker(done) => out.push_str(if done { "[x] " } else { "[ ] " }),
            Event::InlineMath(text) | Event::DisplayMath(text) => out.push_str(&text),
        }
    }

    normalize_spacing(&out)
}

fn heading_prefix(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::H1 => "# ",
        HeadingLevel::H2 => "## ",
        HeadingLevel::H3 => "### ",
        HeadingLevel::H4 => "#### ",
        HeadingLevel::H5 => "##### ",
        HeadingLevel::H6 => "###### ",
    }
}

fn line_start(out: &str) -> bool {
    out.is_empty() || out.ends_with('\n')
}

fn push_blank_line(out: &mut String) {
    if !out.ends_with("\n\n") {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
}

fn normalize_spacing(input: &str) -> String {
    let mut result = String::new();
    let mut blank_count = 0;
    for line in input.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                result.push('\n');
            }
            continue;
        }
        blank_count = 0;
        result.push_str(trimmed);
        result.push('\n');
    }
    result.trim().to_string()
}
