# CommonJS Integration with Module Federation

## Overview

This document explains how CommonJS modules integrate with Rspack's Module Federation system, covering the technical architecture, implementation patterns, and best practices for using CommonJS modules in federated applications.

## Architecture Overview

Module Federation with CommonJS modules involves several layers of integration:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Module Federation Layer                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Host App       │  │  Remote App     │  │  Shared Deps    │ │
│  │  (CommonJS)     │  │  (CommonJS)     │  │  (CommonJS)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────┬───────────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────────┐
│                 CommonJS Integration Layer                      │
│  • require() → Dynamic Import Adaptation                       │
│  • exports/module.exports → Federation Exports                 │
│  • Dependency Sharing → ConsumeShared Mapping                  │
└─────────────────────────┬───────────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────────┐
│                   Rspack Build System                          │
│  • CommonJS Dependency Analysis                                │
│  • Template Rendering with Federation Context                  │
│  • Runtime Code Generation                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Integration Patterns

### 1. CommonJS Module as Remote

Expose CommonJS modules from a remote application:

```javascript
// Remote application - rspack.config.js
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'remote-commonjs',
      filename: 'remoteEntry.js',
      exposes: {
        // Expose CommonJS modules
        './utils': './src/utils.js',
        './data-processor': './src/data-processor.js',
        './legacy-helper': './src/legacy-helper.js'
      },
      shared: {
        // Share CommonJS dependencies
        'lodash': { singleton: true },
        'moment': { singleton: true }
      }
    })
  ]
};
```

```javascript
// Remote CommonJS module - src/utils.js
const _ = require('lodash');
const moment = require('moment');

// Standard CommonJS exports
exports.formatDate = function(date) {
  return moment(date).format('YYYY-MM-DD');
};

exports.processArray = function(arr, transform) {
  return _.map(arr, transform);
};

// Module-level export
module.exports.config = {
  version: '1.0.0',
  features: ['format', 'process']
};
```

### 2. CommonJS Host Consuming Remote

Use CommonJS require to consume federated modules:

```javascript
// Host application - rspack.config.js
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'host-commonjs',
      remotes: {
        'remote-utils': 'remote-commonjs@http://localhost:3001/remoteEntry.js'
      },
      shared: {
        'lodash': { singleton: true },
        'moment': { singleton: true }
      }
    })
  ]
};
```

```javascript
// Host CommonJS module consuming remote
// Asynchronous loading with CommonJS pattern
async function loadRemoteUtils() {
  try {
    // Dynamic import of federated module
    const remoteUtils = await import('remote-utils/utils');
    
    // Use in CommonJS context
    const result = remoteUtils.formatDate(new Date());
    
    // Export from host module
    exports.processWithRemote = function(data) {
      return remoteUtils.processArray(data, item => item.toUpperCase());
    };
  } catch (error) {
    // Fallback implementation
    exports.processWithRemote = function(data) {
      return data.map(item => item.toUpperCase());
    };
  }
}

// Initialize remote loading
loadRemoteUtils();
```

### 3. Mixed Module Systems

Integrate CommonJS with ESM in federated setup:

```javascript
// ESM host consuming CommonJS remote
import { formatDate, processArray } from 'remote-utils/utils';

export function processData(data) {
  const formatted = formatDate(data.timestamp);
  const processed = processArray(data.items, item => ({ ...item, formatted }));
  return processed;
}
```

## Technical Implementation

### ConsumeShared Integration

CommonJS modules in Module Federation use ConsumeShared for dependency sharing:

```rust
// Enhanced ConsumeShared detection for CommonJS
impl CommonJsRequireDependencyTemplate {
  fn detect_consume_shared_context(
    module_graph: &ModuleGraph,
    dep_id: &DependencyId,
    module_identifier: &ModuleIdentifier,
    request: &str,
  ) -> Option<String> {
    // Check if the required module is configured as shared
    if let Some(shared_config) = module_graph.get_shared_config(request) {
      return Some(shared_config.share_key.clone());
    }
    
    // Check parent modules for ConsumeShared context
    for connection in module_graph.get_incoming_connections(module_identifier) {
      if let Some(origin_module) = connection.original_module_identifier.as_ref() {
        if let Some(origin) = module_graph.module_by_identifier(origin_module) {
          if origin.module_type() == &ModuleType::ConsumeShared {
            return origin.get_consume_shared_key();
          }
        }
      }
    }
    
    None
  }
}
```

### Runtime Code Generation

CommonJS modules generate runtime code for federation context:

```javascript
// Generated runtime code for CommonJS federated module
(function(modules) {
  function __webpack_require__(moduleId) {
    // Check if module is shared
    if (__webpack_require__.S && __webpack_require__.S[moduleId]) {
      return __webpack_require__.S[moduleId];
    }
    
    // Standard CommonJS loading
    var module = { exports: {} };
    modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
    return module.exports;
  }
  
  // Federation-specific runtime
  __webpack_require__.S = {}; // Shared modules cache
  __webpack_require__.F = {}; // Federation loading functions
  
  return __webpack_require__;
})({
  './src/utils.js': function(module, exports, __webpack_require__) {
    // Original CommonJS module code with federation context
    const _ = __webpack_require__(/*! lodash */ 'lodash');
    
    exports.formatDate = function(date) {
      return new Date(date).toISOString();
    };
  }
});
```

## Configuration Patterns

### Basic Setup

```javascript
// rspack.config.js for CommonJS + Module Federation
module.exports = {
  entry: './src/index.js',
  mode: 'development',
  plugins: [
    new ModuleFederationPlugin({
      name: 'commonjs-app',
      filename: 'remoteEntry.js',
      
      // Expose CommonJS modules
      exposes: {
        './legacy-utils': './src/legacy-utils.js',
        './data-helpers': './src/data-helpers.js'
      },
      
      // Consume CommonJS remotes
      remotes: {
        'legacy-app': 'legacy-app@http://localhost:3002/remoteEntry.js'
      },
      
      // Share CommonJS dependencies
      shared: {
        'lodash': {
          singleton: true,
          requiredVersion: '^4.17.0'
        },
        'moment': {
          singleton: true,
          requiredVersion: '^2.29.0'
        }
      }
    })
  ]
};
```

### Advanced Configuration

```javascript
// Advanced federation config with CommonJS optimization
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'advanced-commonjs',
      
      // Runtime-specific configurations
      runtime: 'commonjs-runtime',
      
      // Library format for CommonJS compatibility
      library: {
        type: 'commonjs-module'
      },
      
      // Enhanced sharing configuration
      shared: {
        'lodash': {
          singleton: true,
          strictVersion: true,
          requiredVersion: '^4.17.0',
          // CommonJS-specific sharing
          import: 'lodash',
          shareKey: 'lodash',
          shareScope: 'default'
        }
      },
      
      // Expose with CommonJS metadata
      exposes: {
        './utils': {
          import: './src/utils.js',
          name: 'utils'
        }
      }
    })
  ],
  
  // Optimization for CommonJS modules
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        commonjs: {
          test: /node_modules.*\.js$/,
          name: 'commonjs-vendor',
          chunks: 'all'
        }
      }
    }
  }
};
```

## Best Practices

### 1. Module Structure

```javascript
// Good: Clean CommonJS export structure
// src/api-client.js
const axios = require('axios');
const config = require('./config');

class ApiClient {
  constructor(baseURL) {
    this.baseURL = baseURL || config.defaultAPI;
    this.client = axios.create({ baseURL: this.baseURL });
  }

  async get(endpoint) {
    const response = await this.client.get(endpoint);
    return response.data;
  }
}

// Single clear export
module.exports = ApiClient;

// Alternative: Multiple named exports
exports.ApiClient = ApiClient;
exports.createClient = (baseURL) => new ApiClient(baseURL);
```

### 2. Dependency Management

```javascript
// Good: Centralized dependency management
// src/dependencies.js
module.exports = {
  // External dependencies
  lodash: require('lodash'),
  moment: require('moment'),
  axios: require('axios'),
  
  // Internal utilities
  utils: require('./utils'),
  config: require('./config'),
  
  // Federation-aware loading
  async loadRemote(remoteName, moduleName) {
    try {
      const remote = await import(`${remoteName}/${moduleName}`);
      return remote;
    } catch (error) {
      console.warn(`Failed to load remote ${remoteName}/${moduleName}:`, error);
      return null;
    }
  }
};
```

### 3. Error Handling

```javascript
// Robust error handling for federated CommonJS modules
function createFederationWrapper(modulePath) {
  return {
    async load() {
      try {
        const module = await import(modulePath);
        return module;
      } catch (error) {
        console.error(`Failed to load federated module ${modulePath}:`, error);
        return this.getFallback();
      }
    },
    
    getFallback() {
      // Provide fallback implementation
      return {
        default: () => console.warn(`Fallback implementation for ${modulePath}`)
      };
    }
  };
}

// Usage
const remoteUtils = createFederationWrapper('remote-app/utils');
remoteUtils.load().then(utils => {
  exports.processData = utils.processData || (() => 'fallback');
});
```

## Common Challenges and Solutions

### 1. Circular Dependencies

**Problem**: CommonJS circular dependencies in federated modules.

**Solution**: Lazy loading and proper module structure.

```javascript
// Avoid circular dependencies
// Instead of:
const helperB = require('./helper-b'); // helper-b requires helper-a

// Use lazy loading:
function getHelperB() {
  return require('./helper-b');
}

exports.process = function(data) {
  const helper = getHelperB();
  return helper.process(data);
};
```

### 2. Shared Dependency Conflicts

**Problem**: Version conflicts in shared CommonJS dependencies.

**Solution**: Careful version management and fallbacks.

```javascript
// rspack.config.js
shared: {
  'lodash': {
    singleton: true,
    strictVersion: false, // Allow version flexibility
    requiredVersion: '^4.0.0'
  }
}
```

### 3. Runtime Loading Issues

**Problem**: CommonJS `require()` doesn't work with federated modules.

**Solution**: Use dynamic imports with CommonJS compatibility.

```javascript
// Create CommonJS-compatible federation loader
async function requireFederated(remoteName, moduleName) {
  const module = await import(`${remoteName}/${moduleName}`);
  
  // Convert ESM to CommonJS-compatible object
  if (module.default && typeof module.default === 'object') {
    return { ...module.default, ...module };
  }
  
  return module;
}

// Usage
requireFederated('remote-app', 'utils').then(utils => {
  // Use as CommonJS module
  exports.formatData = utils.formatData;
});
```

## Performance Considerations

### Bundle Size Optimization

- **Tree Shaking**: Ensure CommonJS modules support tree shaking
- **Code Splitting**: Split CommonJS modules appropriately  
- **Shared Dependencies**: Maximize shared dependency usage

### Loading Performance

- **Preloading**: Preload critical federated CommonJS modules
- **Caching**: Leverage browser and build-time caching
- **Lazy Loading**: Load non-critical modules on demand

### Runtime Performance

- **Module Caching**: Cache loaded modules properly
- **Memory Management**: Avoid memory leaks in federated context
- **Error Boundaries**: Implement proper error handling

## Migration Strategies

### From Monolith to Federated CommonJS

1. **Identify Module Boundaries**: Analyze existing CommonJS modules
2. **Extract Shared Dependencies**: Identify common dependencies
3. **Create Federation Points**: Define expose/remote configurations  
4. **Gradual Migration**: Migrate modules incrementally
5. **Test Integration**: Ensure proper federation behavior

### From ESM to CommonJS Federation

1. **Module Format Conversion**: Convert ESM to CommonJS where needed
2. **Import/Export Mapping**: Map ESM imports to CommonJS requires
3. **Build Configuration**: Update build tools for mixed modules
4. **Runtime Compatibility**: Ensure runtime compatibility

## Testing Strategies

### Unit Testing

```javascript
// Test CommonJS federated modules
const { ApiClient } = require('../src/api-client');

describe('Federated CommonJS Module', () => {
  test('should create API client', () => {
    const client = new ApiClient('https://api.example.com');
    expect(client.baseURL).toBe('https://api.example.com');
  });
  
  test('should handle federation loading', async () => {
    const mockRemote = { processData: jest.fn() };
    jest.doMock('remote-app/utils', () => mockRemote);
    
    const utils = await import('../src/utils');
    expect(utils.processData).toBeDefined();
  });
});
```

### Integration Testing

```javascript
// Test federated module integration
describe('Module Federation Integration', () => {
  test('should load remote CommonJS module', async () => {
    // Mock federation runtime
    global.__webpack_require__ = {
      S: {}, // Shared modules
      F: {}  // Federation functions
    };
    
    const remoteModule = await import('remote-app/utils');
    expect(remoteModule.formatDate).toBeDefined();
  });
});
```

## Troubleshooting

### Common Issues

1. **Module Not Found**: Check remote URL and exposed module names
2. **Shared Dependency Conflicts**: Verify version compatibility
3. **Runtime Errors**: Check for proper async loading patterns
4. **Build Failures**: Ensure proper CommonJS syntax and exports

### Debug Tools

- **Federation Debug Mode**: Enable detailed logging
- **Module Inspector**: Examine loaded modules and dependencies
- **Network Monitoring**: Check remote module loading
- **Source Maps**: Enable source maps for debugging

For more detailed CommonJS implementation information, see the [CommonJS Documentation](../commonjs/README.md).