<picture>
  <img alt="Rspack Banner" src="https://assets.rspack.rs/rspack/rspack-banner.png">
</picture>

# @rspack/binding-builder-cli

Binding builder cli for rspack custom binding users.

## Rationale

This package is designed to provide a command-line interface for building custom bindings for rspack.

Rspack internally performs post-processing modifications on the generated TypeScript definition (`.d.ts`) files from NAPI-RS bindings. The `@rspack/binding-builder-cli` synchronizes these modifications, ensuring that the generated `.d.ts` files are complete and provide access to all type information available in `@rspack/binding`.

## Installation

```bash
# Install as a dependency
npm install @rspack/binding-builder-cli
```

## Usage

### Basic Usage

```bash
# Basic build
rspack-builder

# Release build
rspack-builder --release

# Build with specific profile
rspack-builder --profile release-debug

# Watch mode for development
rspack-builder --watch
```

### Advanced Options

#### Build Target and Paths

```bash
# Specify build target
rspack-builder --target x86_64-unknown-linux-musl

# Set working directory
rspack-builder --cwd /path/to/project

# Custom Cargo.toml path
rspack-builder --manifest-path /path/to/Cargo.toml

# Custom package.json path
rspack-builder --package-json-path /path/to/package.json
```

#### TypeScript Definition Options

```bash
# Custom .d.ts output path
rspack-builder --dts custom-binding.d.ts

# Custom header for .d.ts file
rspack-builder --dts-header /path/to/header.d.ts

# Disable default .d.ts header
rspack-builder --no-dts-header

# Disable .d.ts cache
rspack-builder --no-dts-cache
```

#### Feature Management

```bash
# Enable specific features
rspack-builder --features plugin,serde

# Enable all features
rspack-builder --all-features

# Disable default features
rspack-builder --no-default-features

# Combine options
rspack-builder --no-default-features --features plugin
```

#### Platform and Output Options

```bash
# Enable platform-specific naming
rspack-builder --platform

# Disable JS binding generation
rspack-builder --no-js

# Generate ESM format
rspack-builder --esm

# Strip binary for minimum size
rspack-builder --strip
```

### Help

View all available options:

```bash
rspack-builder --help
```
