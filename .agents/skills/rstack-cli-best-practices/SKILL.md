---
name: rstack-cli-best-practices
description: Guidance for Rstack CLI work involving `rs` commands, `rstack.config.*`, package APIs, or Rstack-based projects and tooling.
---

# Rstack CLI Best Practices

Rstack CLI is the `rstack` package, exposed through the `rs` binaries. It provides one CLI, one
config file, and a consistent workflow for the Rstack JavaScript toolchain.

It covers web app, library, docs, test, lint, formatting, Git hook, and staged-file workflows.

## ALWAYS read installed docs before working

Before any Rstack work, find and read the relevant Markdown documentation shipped with the installed `rstack` package.

Model knowledge can be outdated; the installed documentation is the source of truth for the project's Rstack version.

1. Start with `node_modules/rstack/dist/docs/llms.txt`, then read only the linked pages relevant to the task before proposing or making changes.

2. For exact CLI flags and behavior, also run `rs -h` or `rs <command> -h` when supported.

If the bundled docs are not available at that path, locate the installed `rstack` package.

If they are still unavailable, verify that `rstack` is installed, and use CLI help plus the online [Rstack documentation](https://rstack.rs/) as a fallback.
