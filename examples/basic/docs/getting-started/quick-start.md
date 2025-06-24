# Quick Start Guide

## Installation

```bash
npm install @rspack/core @rspack/cli --save-dev
```

## Basic Configuration

Create `rspack.config.js`:

```javascript
const path = require('path');

module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  mode: 'development',
};
```

## Your First Build

1. **Create source files:**

```javascript
// src/index.js
import { greet } from './utils.js';

console.log(greet('World'));
```

```javascript
// src/utils.js
export function greet(name) {
  return `Hello, ${name}!`;
}
```

2. **Run the build:**

```bash
npx rspack build
```

3. **View the output in `dist/bundle.js`**

## Next Steps

- [Configuration Basics](configuration-basics.md) - Learn essential configuration patterns
- [Common Patterns](common-patterns.md) - Explore frequently used build patterns
- [Architecture Overview](../architecture/overview.md) - Understand how Rspack works