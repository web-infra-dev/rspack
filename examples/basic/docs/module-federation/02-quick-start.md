# Module Federation Quick Start Guide

A practical getting started guide for Rspack Module Federation with advanced tree-shaking and sharing optimization.

## Table of Contents
- [Basic Setup](#basic-setup)
- [Configuration Examples](#configuration-examples)
- [Step-by-Step Implementation](#step-by-step-implementation)
- [Common Patterns](#common-patterns)
- [Troubleshooting](#troubleshooting)

## Basic Setup

### 1. Install Dependencies

```bash
npm install @rspack/core @rspack/cli
```

### 2. Project Structure

```
my-federated-app/
├── src/
│   ├── index.js                 # Main entry point
│   ├── shared/
│   │   ├── utils.js            # Shared utilities
│   │   ├── components.js       # Shared components
│   │   └── api.js              # Shared API client
│   └── remotes/
├── dist/                       # Build output
└── rspack.config.js           # Rspack configuration
```

### 3. Basic Configuration

```javascript
const rspack = require("@rspack/core");

module.exports = {
  entry: "./src/index.js",
  mode: "development",
  
  // Essential optimization settings for tree-shaking
  optimization: {
    usedExports: true,
    providedExports: true,
    sideEffects: false,
    innerGraph: true,
    mangleExports: true
  },
  
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: "my_app",
      filename: "remoteEntry.js",
      
      // Configure shared modules
      shared: {
        // External libraries
        react: {
          singleton: true,
          requiredVersion: "^18.0.0"
        },
        
        // Local shared modules
        "./shared/utils": {
          singleton: true,
          eager: false,
          shareKey: "utility-lib"
        }
      }
    })
  ]
};
```

## Configuration Examples

### Complete Production Configuration

```javascript
const rspack = require("@rspack/core");

module.exports = {
  context: __dirname,
  entry: {
    main: "./src/index.js"
  },
  mode: "production",
  
  output: {
    clean: true,
    publicPath: "auto"
  },
  
  optimization: {
    minimize: true,
    usedExports: true,
    providedExports: true,
    sideEffects: false,
    innerGraph: true,
    mangleExports: true,
    removeAvailableModules: true,
    removeEmptyChunks: true,
    mergeDuplicateChunks: true,
    realContentHash: true
  },
  
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: "host_app",
      filename: "remoteEntry.js",
      
      // Expose modules to other applications
      exposes: {
        "./Button": "./src/components/Button",
        "./utils": "./src/shared/utils"
      },
      
      // Consume remote modules
      remotes: {
        shell: "shell@http://localhost:3000/remoteEntry.js",
        shared_lib: "shared_lib@http://localhost:3001/remoteEntry.js"
      },
      
      // Share dependencies with version management
      shared: {
        react: {
          singleton: true,
          requiredVersion: "^18.2.0",
          eager: false,
          shareScope: "default"
        },
        "react-dom": {
          singleton: true,
          requiredVersion: "^18.2.0",
          eager: false
        },
        lodash: {
          singleton: true,
          requiredVersion: "^4.17.21",
          eager: false
        },
        
        // Local shared modules
        "./shared/utils": {
          singleton: true,
          eager: false,
          shareKey: "utility-lib"
        },
        "./shared/components": {
          singleton: true,
          eager: false,
          shareKey: "component-lib"
        }
      }
    })
  ]
};
```

### Development Configuration with Debug Output

```javascript
module.exports = {
  mode: "development",
  devtool: "source-map",
  
  optimization: {
    minimize: false, // Disable for debugging
    usedExports: true,
    providedExports: true,
    sideEffects: false
  },
  
  stats: {
    modules: true,
    usedExports: true,
    providedExports: true,
    reasons: true,
    moduleTrace: true,
    optimizationBailout: true
  },
  
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      // ... your configuration
    })
  ]
};
```

## Step-by-Step Implementation

### Step 1: Create Shared Modules

Create reusable modules that can be shared between applications:

```javascript
// src/shared/utils.js
export function formatDate(date) {
  return date.toLocaleDateString();
}

export function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

// Export utility object for tree-shaking analysis
export const utils = {
  formatDate,
  capitalize,
  debounce
};
```

```javascript
// src/shared/components.js
export class Button {
  constructor(text, onClick) {
    this.text = text;
    this.onClick = onClick;
  }
  
  render() {
    const button = document.createElement('button');
    button.textContent = this.text;
    button.onclick = this.onClick;
    return button;
  }
}

export class Modal {
  constructor(title, content) {
    this.title = title;
    this.content = content;
    this.isOpen = false;
  }
  
  open() {
    this.isOpen = true;
    console.log(`Modal "${this.title}" opened`);
  }
  
  close() {
    this.isOpen = false;
    console.log(`Modal "${this.title}" closed`);
  }
}
```

### Step 2: Configure Module Federation

```javascript
// rspack.config.js
const rspack = require("@rspack/core");

module.exports = {
  plugins: [
    new rspack.container.ModuleFederationPlugin({
      name: "my_app",
      filename: "remoteEntry.js",
      
      shared: {
        // Share local modules
        "./shared/utils": {
          singleton: true,
          eager: false,
          shareKey: "utility-lib"
        },
        "./shared/components": {
          singleton: true,
          eager: false,
          shareKey: "component-lib"
        },
        
        // Share external dependencies
        react: {
          singleton: true,
          requiredVersion: "^18.0.0"
        }
      }
    })
  ]
};
```

### Step 3: Use Shared Modules

```javascript
// src/index.js
// Import only what you need for optimal tree-shaking
import { formatDate, capitalize } from "./shared/utils.js";
import { Button } from "./shared/components.js";

// External shared dependencies
import React from "react";
import { map, filter } from "lodash";

// Use the imported modules
console.log("Formatted date:", formatDate(new Date()));
console.log("Capitalized text:", capitalize("hello world"));

const button = new Button("Click me", () => {
  console.log("Button clicked!");
});

// Use external shared dependencies
const data = [1, 2, 3, 4, 5];
const doubled = map(data, n => n * 2);
const filtered = filter(data, n => n > 2);

console.log("Processed data:", { doubled, filtered });
```

### Step 4: Build and Analyze

```bash
# Build the application
npx rspack build

# Analyze the bundle (if you have webpack-bundle-analyzer)
npx webpack-bundle-analyzer dist/main.js
```

## Common Patterns

### 1. Conditional Loading Pattern

```javascript
// Dynamic import with fallback
async function loadRemoteComponent() {
  try {
    const RemoteComponent = await import("remote_app/Component");
    return RemoteComponent.default;
  } catch (error) {
    console.warn("Remote component failed to load, using fallback");
    const FallbackComponent = await import("./components/Fallback");
    return FallbackComponent.default;
  }
}
```

### 2. Version-Safe Sharing Pattern

```javascript
// rspack.config.js
shared: {
  react: {
    singleton: true,
    requiredVersion: "^18.0.0",
    strictVersion: true, // Enforce exact version matching
    eager: false
  }
}
```

### 3. Scoped Sharing Pattern

```javascript
shared: {
  "@company/design-system": {
    singleton: true,
    shareScope: "design",
    shareKey: "design-system"
  },
  "@company/api-client": {
    singleton: true,
    shareScope: "api",
    shareKey: "api-client"
  }
}
```

### 4. Tree-Shaking Optimization Pattern

```javascript
// Optimal import pattern - import only what you need
import { debounce, throttle } from "lodash-es";
// NOT: import _ from "lodash"; // This imports everything

// For local modules, use named exports
export { formatDate, capitalize }; // Good for tree-shaking
// NOT: export default { formatDate, capitalize }; // Harder to tree-shake
```

### 5. Error Boundary Pattern

```javascript
// Error boundary for remote modules
class RemoteErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false };
  }
  
  static getDerivedStateFromError(error) {
    return { hasError: true };
  }
  
  componentDidCatch(error, errorInfo) {
    console.error("Remote module error:", error, errorInfo);
  }
  
  render() {
    if (this.state.hasError) {
      return <div>Something went wrong loading the remote module.</div>;
    }
    return this.props.children;
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Module Not Found Error

**Problem**: `Module not found: Error: Can't resolve 'remote_app/Component'`

**Solution**:
```javascript
// Check remote configuration
remotes: {
  remote_app: "remote_app@http://localhost:3001/remoteEntry.js"
}

// Ensure the remote is running and accessible
// Verify the exposed module name matches
```

#### 2. Version Conflicts

**Problem**: Different versions of shared dependencies

**Solution**:
```javascript
shared: {
  react: {
    singleton: true,
    requiredVersion: "^18.0.0",
    strictVersion: false, // Allow compatible versions
    fallback: false // Don't bundle fallback
  }
}
```

#### 3. Shared Module Not Loading

**Problem**: Shared modules are bundled instead of shared

**Solution**:
```javascript
// Ensure consistent shareKey across applications
shared: {
  "./shared/utils": {
    singleton: true,
    shareKey: "utility-lib", // Must match across apps
    shareScope: "default"    // Must match across apps
  }
}
```

#### 4. Tree-Shaking Not Working

**Problem**: Unused exports are not eliminated

**Solution**:
```javascript
// Enable proper optimization settings
optimization: {
  usedExports: true,
  providedExports: true,
  sideEffects: false, // Mark package as side-effect free
  innerGraph: true,
  mangleExports: true
}

// In package.json
{
  "sideEffects": false // or ["*.css", "*.scss"]
}
```

#### 5. Runtime Loading Errors

**Problem**: `Loading script failed` or `ChunkLoadError`

**Solution**:
```javascript
// Add error handling for dynamic imports
async function loadWithRetry(importFn, retries = 3) {
  for (let i = 0; i < retries; i++) {
    try {
      return await importFn();
    } catch (error) {
      if (i === retries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
}

// Usage
const Component = await loadWithRetry(() => import("remote_app/Component"));
```

### Debug Tips

1. **Enable Verbose Stats**:
```javascript
stats: {
  modules: true,
  usedExports: true,
  providedExports: true,
  reasons: true,
  moduleTrace: true
}
```

2. **Check Network Tab**: Verify remote entries are loading correctly

3. **Use Browser DevTools**: Check for console errors and network failures

4. **Analyze Bundle**: Use tools like webpack-bundle-analyzer to understand what's included

5. **Test Isolation**: Test each remote module independently

### Performance Optimization

1. **Minimize Eager Loading**: Use `eager: false` for non-critical shared modules
2. **Optimize Chunk Splitting**: Configure `splitChunks` for better caching
3. **Use CDN**: Host shared dependencies on CDN when possible
4. **Monitor Bundle Size**: Regularly analyze bundle size and shared module usage

This quick start guide provides the foundation for implementing Module Federation with Rspack. For advanced features and detailed analysis, refer to the complete documentation in this directory.