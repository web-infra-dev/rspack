# Rspack Documentation

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

This website is built with [Rspress](https://rspress.dev), the document content can be written using markdown or mdx syntax. You can refer to the [Rspress Website](https://rspress.dev) for detailed usage.

The source code of Rspress can be found in [this folder](https://github.com/web-infra-dev/rspress).

If you have any problems using the Rspress, please create a new issue at [Rspress Issues](https://github.com/web-infra-dev/rspress/issues).

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
