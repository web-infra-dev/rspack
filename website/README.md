# Rspack documentation

📄 Documentation for Rspack.

## Translation

Currently Rspack provides documentation in English and Chinese. If you can use Chinese, please update both documents at the same time. Otherwise, just update the English documentation.

```bash
root
└─ docs
   ├─ en     # English Document
   └─ zh     # Chinese Document
```

## Contributing

This website is built with [Rspress](https://rspress.rs), the document content can be written using markdown or mdx syntax. You can refer to the [Rspress Website](https://rspress.rs) for detailed usage.

The source code of Rspress can be found in [this folder](https://github.com/web-infra-dev/rspress).

If you have any problems using the Rspress, please create a new issue at [Rspress Issues](https://github.com/web-infra-dev/rspress/issues).

## Writing style guide

- **Capitalization style**: page titles and section headings should use sentence-style capitalization (only capitalize the first word and proper nouns) rather than title-style capitalization:
  - Correct: "A new method for creating JavaScript rollovers"
  - Incorrect: "A New Method for Creating JavaScript Rollovers"

## Image assets

For images you use in the document, it's better to upload them to the [rstackjs/rstack-design-resources](https://github.com/rstackjs/rstack-design-resources) repository, so the size of the current repository doesn't get too big.

After you upload the images there, they will be automatically deployed under the <https://assets.rspack.rs/>.

### Install dependencies

Enable [pnpm](https://pnpm.io/) with corepack:

```sh
corepack enable
```

Install dependencies:

```sh
pnpm install
```

### Local development

```bash
pnpm install
pnpm run dev
```

### Production build

```bash
pnpm run build
```
