# rspack_binding_api

Shared binding API for Rspack, providing bridge interfaces between Rspack core functionality and Node.js/browser environments.

## Overview

`rspack_binding_api` is the core binding layer in the Rspack project, responsible for exposing Rspack core functionality written in Rust to JavaScript/TypeScript environments. It provides complete API interfaces for compilation, building, module processing, and other functionalities.

## Features

- `browser`: Enable browser environment support
- `debug_tool`: Enable debug tools
- `plugin`: Enable SWC plugin support
- `sftrace-setup`: Enable performance tracing setup

## Important Notice

⚠️ **Version Compatibility Warning**

**This repository does not follow Semantic Versioning (SemVer).**

- Any version update may contain breaking changes
- It is recommended to lock specific version numbers in production environments
- Please thoroughly test all functionality before upgrading

## API Usage Warning

🚨 **This package's API should NOT be used as a public Rust API**

This crate is designed to be linked as a **C dynamic library** during Rspack binding generation, not as a public Rust API for external consumption.

### For Developers

If you're working on Rspack itself:

- This crate is safe to use within the Rspack project
- Changes should be coordinated with the binding generation process
- Test thoroughly when making changes

If you're an external developer:

- Do not depend on this crate directly
- Use the official Rspack Node.js package instead
- Report issues through the main Rspack repository

If you're a user of Rspack custom binding:

- Do not depend on this crate directly
- Use [`rspack_binding_builder`](https://crates.io/crates/rspack_binding_builder) to build your own binding
