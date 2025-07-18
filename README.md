# cargo-onedoc

[![Crates.io Version](https://badgers.space/crates/version/cargo-onedoc)](https://crates.io/crates/cargo-onedoc)
[![Docs.rs Latest](https://badgers.space/badge/docs.rs/latest/blue)](https://docs.rs/cargo-onedoc)
[![Build Status](https://badgers.space/github/checks/rossmacarthur/cargo-onedoc?label=build)](https://github.com/rossmacarthur/cargo-onedoc/actions/workflows/build.yaml)

📝 Generate README.md from doc comments.

Only write your documentation once! This crate provides a Cargo subcommand that
can generate Markdown files from your Rust doc comments.

## 🚀 Getting started

First, install the tool using cargo

```sh
cargo install cargo-onedoc
```

Then generate your README from your `lib.rs` using the following.

```sh
cargo onedoc
```

This will generate a README using the default template. See [config](#config)
for how to configure how files are generated.

## 🌟 Features

This tool can take one or more Markdown files and/or Rust source files and
output a single Markdown file.

When converting between Rustdoc Markdown and CommonMark, the following changes
are made.

### Headings

Headings are increased by one level. E.g. `#` becomes `##`. For example:

```rust
//! ## MSRV
//!
//! Currently the minimum supported version is 1.51.0.
```

Will become

```markdown
### MSRV

Currently the minimum supported version is 1.51.0.
```

### Codeblocks

Bare codeblocks are fenced as Rust codeblocks e.g. `` ```rust ``. Leading `#`
comments from codeblocks are removed.  For example the following doc comment

````rust
//! ```
//! # fn main() {
//! println!("Hello, world!");
//! # }
//! ```
````

Will become

````markdown
```rust
println!("Hello, world!");
```
````

### Intradoc links

Intra doc links are converted based on the the `links` section of the config.
For example assuming the following config:

```toml
[links]
"String" = "https://doc.rust-lang.org/stable/std/string/struct.String.html"
```

The following doc comment

```rust
//! Render the template to a [`String`].
```

Will become

```markdown
Render the template to a [`String`][String].

[String]: https://doc.rust-lang.org/stable/std/string/struct.String.html
```

## 👷 Config

This tool can be configured using a `onedoc.toml` file. There are three main
sections `badges`, `doc` and `links`.

### `badges`

The badges settings are only relevant if you use the default README template.

```toml
[badges]
crates_io = true  # whether to add a badge for the crates.io page
docs_rs = true    # whether to add a badge for the docs.rs page

# optional badge for a github workflow
# name should be the name of the github workflow
github_workflow = { label = "build", name = "build" }
```

### `doc`

The `doc` section is used to specify the input files and the output file. The
`input` field is a list of files to read. The `output` field is the file to
write to. The `template` field is the template file to use.

Here is an example from the
[`sheldon`](https://github.com/rossmacarthur/sheldon) repository. This
constructs a `README.md` using multiple Markdown files.

```toml
[[doc]]
input = [
    "docs/src/Installation.md",
    "docs/src/Getting-started.md",
    "docs/src/Command-line-interface.md",
    "docs/src/Configuration.md",
]
output = "README.md"
template = "docs/README_TEMPLATE.md"
```

Here is another example from the [`upon`](https://github.com/rossmacarthur/upon)
repository.

```toml
[[doc]]
input = "src/syntax.rs"
output = "SYNTAX.md"
template = "docs/SYNTAX_TEMPLATE.md"
```

This constructs `SYNTAX.md` from the given Rust module and provided template.

### `links`

The `links` is used to specific intra doc link mapping. This is needed because
this tool does not figure out the correct link to use. This is simply a mapping
of the link text to the URL.

```toml
[links]
"Display" = "https://doc.rust-lang.org/stable/std/fmt/trait.Display.html"
```

## License

This project is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
