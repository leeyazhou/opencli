use criterion::{criterion_group, criterion_main, Criterion};

fn bench_markdown_render(c: &mut Criterion) {
    let input = include_str!("../tests/fixtures/markdown_sample.md");
    c.bench_function("markdown_render_fixture", |b| {
        b.iter(|| {
            let _ = render_for_bench(input);
        })
    });
}

fn render_for_bench(input: &str) -> String {
    use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
    let mut out = String::new();
    let mut list_stack: Vec<Option<u64>> = Vec::new();
    let mut in_blockquote = false;
    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    for event in Parser::new_ext(input, Options::all()) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => out.push_str(match level {
                    HeadingLevel::H1 => "# ",
                    HeadingLevel::H2 => "## ",
                    HeadingLevel::H3 => "### ",
                    HeadingLevel::H4 => "#### ",
                    HeadingLevel::H5 => "##### ",
                    HeadingLevel::H6 => "###### ",
                }),
                Tag::BlockQuote(_) => in_blockquote = true,
                Tag::List(start) => list_stack.push(start),
                Tag::Item => match list_stack.last_mut() {
                    Some(Some(n)) => {
                        out.push_str(&format!("{}. ", *n));
                        *n += 1;
                    }
                    _ => out.push_str("- "),
                },
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.into_string()),
                        _ => None,
                    };
                    out.push_str("```\n");
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    code_lang = None;
                    out.push_str("```\n");
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_blockquote {
                    out.push_str("> ");
                    in_blockquote = false;
                }
                if in_code_block && let Some(lang) = code_lang.take() {
                    out.push_str(&format!("# language: {lang}\n"));
                }
                out.push_str(&text);
                out.push('\n');
            }
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            _ => {}
        }
    }
    out
}

criterion_group!(benches, bench_markdown_render);
criterion_main!(benches);
