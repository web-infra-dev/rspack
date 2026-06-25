---
name: release-blog-writer
description: Write or revise release blog posts for product releases. Use when the user asks to draft, polish, restructure, or adapt a release announcement.
---

# Release blog writer

## Purpose

Use this skill to write release blogs that feel like product communication, not raw changelog output. The blog may be written in English, Chinese, or another user-requested language, but these instructions stay language-agnostic and project-agnostic.

## Core writing rules

- Follow the user's requested language and the style of existing release blogs for the same product.
- Use previous major or minor release posts as the primary style reference when available.
- Prefer clear user value over implementation trivia.
- Avoid turning the blog into a complete changelog.

## Recommended shape

Use the project's existing blog metadata and component conventions. If the product has neighboring release posts, mirror their frontmatter, author block, date format, title style, and navigation pattern.

## Section writing

Each feature section should usually follow this pattern:

- Describe the change clearly so readers understand its background, purpose, use cases, and user-facing benefits such as performance gains.
- Avoid exhaustive API lists; select the key APIs or behaviors readers need, and when useful, use one precise code or configuration example to make the change clear at a glance.

## Headings

- Use sentence case for titles and headings.
- Keep headings short and specific.
- Keep anchors stable and readable.
  - If the website is built with Rspress, use the Rspress anchor syntax, such as `\{#foo-bar}`.

## Tone

- Be professional, direct, and understated.
- Avoid vague claims such as "greatly improved" unless the improvement is explained or quantified.
- Avoid slogan-like positioning in feature updates unless intentionally discussing product vision.
- Qualify benchmark numbers with enough context to make them credible.
- When presenting performance gains, bold only the key improvement numbers, avoid frequent emphasis, and use tables or images when they make the comparison clearer at a glance.
- Use product and ecosystem terms consistently.
- Write naturally in the target language; do not let the prose read like a literal translation from another language.

## Links

Add links where they help readers continue:

- Product documentation for options, APIs, plugins, or migration steps.
- Upstream PRs or specs for implementation context.
- Platform documentation for browser, Node.js, or language features.

Do not over-link common terms. Link the first meaningful occurrence or the option name users are likely to search for.

## Revision behavior

When revising an existing blog:

- Keep the user's latest manual edits unless they conflict with the request.
- Prefer small targeted edits over broad rewrites.
- If the user asks to remove a kind of detail, remove it consistently across the post.
