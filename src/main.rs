//! 📝 Generate README.md from doc comments.
//!
//! Only write your documentation once! This crate provides a Cargo subcommand
//! that can generate Markdown files from your Rust doc comments.

mod config;
mod fix;
mod render;

use std::collections::BTreeMap;
use std::fs;
use std::io;

use anyhow::{anyhow, bail, Context as _, Result};
use camino::Utf8Path as Path;
use cargo_metadata::Package;
use clap::Parser as _;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_toc as toc;

use crate::config::{Config, Doc};

#[derive(Debug, clap::Parser)]
#[clap(
    name = "cargo",
    bin_name = "cargo",
    disable_help_subcommand(true),
    subcommand_required(true)
)]
enum Cargo {
    #[clap(name = "onedoc")]
    Command(Opt),
}

#[derive(Debug, Clone, clap::Args)]
#[command(author, version, about)]
struct Opt {
    #[clap(long, short)]
    package: Option<String>,

    #[clap(long)]
    check: bool,
}

pub struct Context<'a> {
    check: bool,
    package: &'a Package,
    config: Config,
}
fn main() -> Result<()> {
    let Cargo::Command(Opt { check, package }) = Cargo::parse();
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;

    let pkg = match package {
        Some(name) => metadata
            .workspace_packages()
            .into_iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow!("package `{}` not found in workspace", name))?,
        None => metadata.root_package().context("no root package")?,
    };

    let config = config::load(&metadata, pkg)?;
    generate_all(Context {
        check,
        package: pkg,
        config,
    })
}

fn generate_all(ctx: Context<'_>) -> Result<()> {
    let mut engine = {
        let mut e = upon::Engine::new();
        e.add_filter("trim_prefix", |s: &str, p: &str| {
            s.trim_start_matches(p).to_owned()
        });
        e
    };

    for doc in &ctx.config.docs {
        generate_doc(&mut engine, &ctx, doc)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Kind {
    RustDoc,
    Markdown,
}

type Links = BTreeMap<String, Vec<String>>;

fn generate_doc(engine: &mut upon::Engine<'_>, ctx: &Context<'_>, doc: &Doc) -> Result<()> {
    // Compile the template
    let template_name = match &doc.template {
        Some(path) => {
            let contents = fs::read_to_string(path)?;
            engine
                .add_template(path.to_string(), contents)
                .map_err(|e| anyhow!("{:#}", e))?;
            path.to_string()
        }
        None => {
            let name = "<anonymous>";
            engine
                .add_template(name, include_str!("README_TEMPLATE.md"))
                .map_err(|e| anyhow!("{:#}", e))?;
            name.to_string()
        }
    };

    // Load the Markdown to process
    let to_process = {
        let mut items = Vec::new();
        for input in &doc.inputs {
            let item = match input.extension() {
                Some("rs") => {
                    let kind = Kind::RustDoc;
                    let text = get_module_comment(input)
                        .with_context(|| format!("failed to read from `{}`", input))?;
                    (kind, text)
                }
                Some("md") => {
                    let kind = Kind::Markdown;
                    let text = fs::read_to_string(input)
                        .with_context(|| format!("failed to read from `{}`", input))?;
                    (kind, text)
                }
                Some(_) | None => {
                    bail!("unsupported file extension `{}`", input);
                }
            };
            items.push(item);
        }
        items
    };

    let rendered = render(engine, ctx, &template_name, to_process)?;

    let current = match fs::read_to_string(&doc.output) {
        Ok(c) => c,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).context("failed to read current README")?,
    };

    if current == rendered {
        println!("{} is up to date", &doc.output);
    } else if ctx.check {
        bail!("{} is out of date", &doc.output);
    } else {
        fs::write(&doc.output, rendered)
            .with_context(|| format!("failed to write to `{}`", &doc.output))?;
        println!("{} was updated", &doc.output);
    }

    Ok(())
}

fn get_module_comment(path: &Path) -> Result<String> {
    let contents = fs::read_to_string(path)?;
    let lines: Vec<_> = contents
        .lines()
        .take_while(|line| line.starts_with("//!"))
        .map(|line| line.trim_start_matches("//! ").trim_start_matches("//!"))
        .collect();
    Ok(lines.join("\n"))
}

fn render(
    engine: &upon::Engine<'_>,
    ctx: &Context,
    template_name: &str,
    to_process: Vec<(Kind, String)>,
) -> Result<String> {
    let mut events = Vec::new();
    let mut link_config = Links::new();

    for (kind, text) in &to_process {
        let mut es = Vec::from_iter(Parser::new_ext(text, Options::all()));
        // common fixes
        es = fix::headings(es);
        match kind {
            Kind::RustDoc => {
                es = fix::code_blocks(es).context("failed to fix codeblocks")?;
                es = fix::doc_links(ctx, &mut link_config, es);
            }
            Kind::Markdown => {
                es = fix::rel_links(ctx, es);
            }
        }
        events.extend(es);
    }

    // Now render contents as markdown
    let full_contents = render::to_cmark(&events).context("failed to render contents")?;

    let (summary, contents) = {
        let (s, c) = fix::summary(events);
        let summary = render::to_cmark(&s).context("failed to render summary")?;
        let contents = render::to_cmark(&c).context("failed to render contents")?;
        (summary, contents)
    };

    let toc = toc::TableOfContents::new(&full_contents)
        .to_cmark_with_options(toc::Options::default().levels(2..=6));

    let mut rendered = engine
        .get_template(template_name)
        .unwrap()
        .render(upon::value! {
            manifest: ctx.package,
            summary: summary,
            contents: contents,
            full_contents: full_contents,
            toc: toc,
        })
        .map_err(|e| anyhow!("{:#}", e))?;

    // Append link info
    if !link_config.is_empty() {
        rendered.push_str("\n\n");
        for (name, links) in link_config {
            for (i, u) in links.into_iter().enumerate() {
                let name = if i == 0 {
                    name.to_owned()
                } else {
                    format!("{}-{}", name, i)
                };
                rendered = rendered.replace(&format!("({})", name), &format!("[{}]", name));
                rendered.push_str(&format!("[{}]: {}\n", name, u));
            }
        }
    }

    Ok(rendered)
}
