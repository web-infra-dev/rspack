# Rspack Documentation

## Overview

Comprehensive documentation for Rspack's build system, featuring detailed analysis of the export system, Module Federation, CommonJS support, and architectural patterns.

## Quick Navigation

### 🚀 Getting Started
- **[Quick Start Guide](getting-started/quick-start.md)** - Get up and running with Rspack
- **[Configuration Basics](getting-started/configuration-basics.md)** - Essential configuration patterns
- **[Common Patterns](getting-started/common-patterns.md)** - Frequently used build patterns

### 🏗️ Architecture
- **[System Overview](architecture/overview.md)** - High-level architectural concepts
- **[Compilation Pipeline](architecture/compilation-pipeline.md)** - Build process flow
- **[Module Graph](architecture/module-graph.md)** - Module relationship system
- **[Dependency System](architecture/dependency-system.md)** - Dependency resolution and processing

### 🎯 Features
- **[Module Federation](features/module-federation/overview.md)** - Micro-frontend architecture support
- **[CommonJS Support](features/commonjs/overview.md)** - Legacy module system compatibility
- **[Tree Shaking](features/tree-shaking.md)** - Dead code elimination
- **[Code Generation](features/code-generation.md)** - Template system and output generation

### ⚡ Performance
- **[Optimization Guide](performance/overview.md)** - Build performance strategies
- **[Caching Systems](performance/caching.md)** - Multi-level caching implementation
- **[Parallel Processing](performance/parallel-processing.md)** - Concurrency and parallelization
- **[Profiling & Debugging](performance/profiling-debugging.md)** - Performance analysis tools

### 🔧 Implementation Details
- **[Dependency Lifecycle](implementation/dependency-lifecycle.md)** - Complete dependency processing flow
- **[Template System](implementation/template-system.md)** - Code generation and rendering
- **[Init Fragments](implementation/init-fragments.md)** - Module initialization system
- **[Rust Internals](implementation/rust-internals.md)** - Low-level implementation details

### 📚 Reference
- **[API Reference](reference/api-reference.md)** - Complete API documentation
- **[Configuration Schema](reference/configuration-schema.md)** - All configuration options
- **[Troubleshooting](reference/troubleshooting.md)** - Common issues and solutions
- **[Migration Guide](reference/migration-guide.md)** - Upgrading from other bundlers

## Documentation Philosophy

This documentation is organized by user intent and complexity level:

- **Getting Started**: For new users who need to accomplish basic tasks
- **Architecture**: For understanding system design and concepts
- **Features**: For implementing specific functionality
- **Performance**: For optimization and scaling
- **Implementation**: For contributors and advanced debugging
- **Reference**: For quick lookup of specific information

## Contributing

Documentation improvements are welcome. When contributing:

1. Follow the established structure and naming conventions
2. Keep content focused and avoid cross-topic overlap
3. Include practical examples and clear explanations
4. Update cross-references when moving or renaming content

---

*Last updated: This documentation represents the current state of Rspack's export system and related features.*