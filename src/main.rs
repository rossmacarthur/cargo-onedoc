//! 📝 Generate README.md from doc comments.
//!
//! Only write your documentation once! This crate provides a Cargo subcommand
//! that can generate Markdown files from your Rust doc comments.

mod config;
mod fix;
mod render;

use std::fs;
use std::io;

use anyhow::{anyhow, Context as _, Result};
use camino::{Utf8Path as Path, Utf8PathBuf as PathBuf};
use cargo_metadata::{Metadata, Package};
use clap::Parser as _;
use pulldown_cmark::{Options, Parser};
use pulldown_cmark_toc as toc;

use crate::config::{Config, File};

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

#[derive(Debug, clap::Args)]
#[command(author, version, about)]
struct Opt {
    #[clap(long)]
    check: bool,
}

pub struct Context {
    check: bool,
    config: Config,
    metadata: Metadata,
    manifest_dir: PathBuf,
}

impl Context {
    fn package(&self) -> Result<&Package> {
        self.metadata.root_package().context("no root package")
    }
}

fn main() -> Result<()> {
    let Cargo::Command(Opt { check }) = Cargo::parse();

    let mut engine = upon::Engine::new();
    engine.add_filter("trim_prefix", |s: &str, p: &str| {
        s.trim_start_matches(p).to_owned()
    });

    let metadata = cargo_metadata::MetadataCommand::new().exec()?;

    let config_path = metadata.workspace_root.join("onedoc.toml");
    let mut config = match fs::read_to_string(config_path) {
        Ok(contents) => toml::from_str(&contents)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => Config::default(),
        Err(err) => return Err(err).context("failed to read current README")?,
    };

    let manifest_dir = {
        let pkg = metadata.root_package().context("no root package")?;
        pkg.manifest_path.parent().unwrap().to_owned()
    };

    for file in &mut config.files {
        file.input = manifest_dir.join(&file.input);
        file.output = manifest_dir.join(&file.output);
    }

    let ctx = Context {
        check,
        config,
        metadata,
        manifest_dir,
    };

    if ctx.config.files.is_empty() {
        let pkg = ctx.metadata.root_package().context("no root package")?;
        let input = input_path(&pkg)?;
        let output = pkg.readme().context("no readme path in package manifest")?;
        let file = File {
            input,
            output,
            template: None,
        };
        generate(&ctx, &mut engine, &file)?;
    } else {
        for file in &ctx.config.files {
            generate(&ctx, &mut engine, file)?;
        }
    }

    Ok(())
}

fn generate(ctx: &Context, engine: &mut upon::Engine, file: &File) -> Result<()> {
    let pkg = ctx.package()?;

    let name = match &file.template {
        Some(path) => {
            let full = ctx.manifest_dir.join(path);
            let contents = fs::read_to_string(full)?;
            engine.add_template(path.to_string(), contents)?;
            path.to_string()
        }
        None => {
            let name = "<anonymous>";
            engine.add_template(name, include_str!("README_TEMPLATE.md"))?;
            name.to_string()
        }
    };

    engine.add_template("readme", include_str!("README_TEMPLATE.md"))?;

    let text = get_module_comment(&file.input)
        .with_context(|| format!("failed to read from `{}`", &file.input))?;
    let mut events = Vec::from_iter(Parser::new_ext(&text, Options::all()));

    // apply fixups
    events = fix::headings(events);
    events = fix::code_blocks(events).context("failed to fix codeblocks")?;
    let (events, url_config) = fix::links(ctx, events);
    let (summary, events) = fix::summary(events);

    // render contents as markdown
    let summary = render::to_cmark(&summary).context("failed to render summary")?;
    let contents = render::to_cmark(&events).context("failed to render contents")?;

    let toc = toc::TableOfContents::new(&contents)
        .to_cmark_with_options(toc::Options::default().levels(2..=6));

    let mut rendered = engine
        .get_template(&name)
        .unwrap()
        .render(upon::value! {
            manifest: pkg,
            summary: summary,
            contents: contents,
            toc: toc,
        })
        .map_err(|e| anyhow!("{:#}", e))?;

    // Append link info
    if !url_config.is_empty() {
        rendered.push_str("\n\n");
        for (name, urls) in url_config {
            for (i, u) in urls.into_iter().enumerate() {
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

    let current = match fs::read_to_string(&file.output) {
        Ok(c) => c,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).context("failed to read current README")?,
    };

    if current == rendered {
        println!(
            "{} -> {} is up to date",
            file.input.strip_prefix(&ctx.manifest_dir).unwrap(),
            file.output.strip_prefix(&ctx.manifest_dir).unwrap(),
        );
    } else if ctx.check {
        println!(
            "{} -> {} is out of date",
            file.input.strip_prefix(&ctx.manifest_dir).unwrap(),
            file.output.strip_prefix(&ctx.manifest_dir).unwrap(),
        );
    } else {
        fs::write(&file.output, rendered)
            .with_context(|| format!("failed to write to `{}`", &file.output))?;
        println!(
            "{} -> {} was updated",
            file.input.strip_prefix(&ctx.manifest_dir).unwrap(),
            file.output.strip_prefix(&ctx.manifest_dir).unwrap(),
        );
    }

    Ok(())
}

fn input_path(pkg: &Package) -> Result<PathBuf> {
    if let Some(t) = pkg
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| k == "lib"))
    {
        return Ok(t.src_path.clone());
    }

    if let Some(t) = pkg
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| k == "bin"))
    {
        return Ok(t.src_path.clone());
    }

    Err(anyhow!(
        "failed to determine default source file for package"
    ))
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
