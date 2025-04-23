# {{ manifest.name }}

{% if config.badges.crates_io -%}
[![Crates.io Version](https://badgers.space/crates/version/{{ manifest.name }})](https://crates.io/crates/{{ manifest.name }})
{% endif -%}

{% if config.badges.docs_rs -%}
[![Docs.rs Latest](https://badgers.space/badge/docs.rs/latest/blue)](https://docs.rs/{{ manifest.name }})
{% endif -%}

{% if config.badges.github_workflow -%}
[![Build Status](https://badgers.space/github/checks/{{ manifest.repository | trim_prefix: "https://github.com/" }}?label={{ config.badges.github_workflow.label }})]({{ manifest.repository }}/actions/workflows/{{ config.badges.github_workflow.name }}.yaml)
{%- endif %}

{{ summary }}

{{ contents }}

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
