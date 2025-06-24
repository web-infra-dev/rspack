# Introduction to Rspack Module Federation

## Table of Contents

1. [What is Module Federation?](#what-is-module-federation)
2. [Why Use Module Federation?](#why-use-module-federation)
3. [Key Concepts](#key-concepts)
   - [ConsumeShared](#consumeshared)
   - [ProvideShared](#provideshared)
   - [Share Scopes](#share-scopes)
4. [Basic Architecture Overview](#basic-architecture-overview)
5. [Module Federation in Micro-Frontend Architecture](#module-federation-in-micro-frontend-architecture)
6. [Getting Started](#getting-started)
7. [Common Use Cases](#common-use-cases)
8. [Best Practices](#best-practices)

## What is Module Federation?

Module Federation is a powerful feature in Rspack (and Webpack 5) that enables JavaScript applications to dynamically share code and dependencies at runtime. Think of it as a way to build applications that can load and share modules from other independently deployed applications.

In simpler terms, Module Federation allows you to:
- Split your application into smaller, independently deployable pieces
- Share common dependencies between applications without duplicating code
- Load remote modules at runtime as if they were local modules
- Build truly distributed applications with seamless integration

### A Real-World Analogy

Imagine you're building a large office complex. Instead of constructing one massive building, you create several smaller buildings that share utilities like electricity, water, and HVAC systems. Each building can be built, maintained, and updated independently, but they all work together as one cohesive complex. That's Module Federation for web applications!

## Why Use Module Federation?

### 1. **Independent Deployments**
Different teams can work on different parts of an application and deploy them independently without coordinating releases.

```javascript
// Team A's application
exposes: {
  './Header': './src/components/Header',
}

// Team B's application can use Team A's Header
remotes: {
  teamA: 'teamA@http://localhost:3001/remoteEntry.js',
}
```

### 2. **Code Sharing Without Duplication**
Share common libraries (like React, lodash, or custom utilities) across applications without bundling them multiple times.

```javascript
// Both applications share React without duplicating it
shared: {
  react: { singleton: true, requiredVersion: '^18.0.0' },
  'react-dom': { singleton: true, requiredVersion: '^18.0.0' },
}
```

### 3. **Dynamic Module Loading**
Load modules at runtime based on user actions or routes, improving initial load times.

```javascript
// Load a remote module only when needed
const RemoteButton = React.lazy(() => import('remoteApp/Button'));
```

### 4. **Micro-Frontend Architecture**
Build large-scale applications as a composition of smaller, focused applications.

## Key Concepts

### ConsumeShared

ConsumeShared is the mechanism that allows an application to use shared modules provided by other applications. It's like a smart import system that:

- Checks if a shared module is already available in the runtime
- Uses the existing version if compatible
- Loads its own version if necessary

**How it works:**

```javascript
// Configuration
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      shared: {
        // This app wants to consume lodash
        lodash: {
          requiredVersion: '^4.17.0',
          singleton: true,
        },
      },
    }),
  ],
};

// In your code
import _ from 'lodash'; // ConsumeShared handles this intelligently
```

**Behind the scenes:**
1. ConsumeShared checks the share scope for lodash
2. If found and version compatible, uses the shared version
3. If not found or incompatible, loads its own version
4. Ensures only one instance exists if `singleton: true`

### ProvideShared

ProvideShared is the counterpart to ConsumeShared. It makes modules available for other applications to consume.

**How it works:**

```javascript
// Application providing shared modules
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'providerApp',
      shared: {
        // This app provides React to others
        react: {
          eager: true, // Load immediately, not lazily
          singleton: true,
          requiredVersion: '^18.0.0',
        },
        // Custom utility module
        './utils': {
          import: './src/shared/utils',
          requiredVersion: '^1.0.0',
        },
      },
    }),
  ],
};
```

**Key ProvideShared options:**
- `eager`: Load the module immediately instead of on-demand
- `singleton`: Ensure only one instance exists across all apps
- `requiredVersion`: Version constraint for compatibility
- `import`: Path to the module to share

### Share Scopes

A Share Scope is like a global registry where shared modules live. It's the runtime container that manages all shared dependencies across federated applications.

**Key characteristics:**

1. **Global by default**: All applications on the same page share the same default scope
2. **Named scopes**: You can create isolated sharing contexts
3. **Version negotiation**: Automatically selects the best version when multiple exist

```javascript
// Default share scope (most common)
shared: ['react', 'react-dom']

// Custom share scope
shareScope: 'myCustomScope',
shared: {
  react: { 
    shareScope: 'myCustomScope',
    singleton: true 
  }
}
```

**Share Scope in action:**

```javascript
// App A loads first, provides React 18.2.0
// App B loads second, wants React ^18.0.0
// Share scope negotiation: App B uses App A's React (compatible)

// App C loads third, requires React 17.0.0
// Share scope negotiation: App C loads its own React (incompatible)
```

## Basic Architecture Overview

### Components of Module Federation

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Runtime                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌───────────────┐ │
│  │   Host App      │  │  Remote App 1  │  │ Remote App 2  │ │
│  │                │  │                │  │               │ │
│  │ ┌────────────┐ │  │ ┌────────────┐ │  │ ┌───────────┐ │ │
│  │ │   Local    │ │  │ │  Exposed   │ │  │ │  Exposed  │ │ │
│  │ │  Modules   │ │  │ │  Modules   │ │  │ │  Modules  │ │ │
│  │ └────────────┘ │  │ └────────────┘ │  │ └───────────┘ │ │
│  │                │  │                │  │               │ │
│  │ ┌────────────┐ │  │ ┌────────────┐ │  │ ┌───────────┐ │ │
│  │ │  Remotes   │ │  │ │   Shared   │ │  │ │  Shared   │ │ │
│  │ │  Config    │ │  │ │  Modules   │ │  │ │  Modules  │ │ │
│  │ └────────────┘ │  │ └────────────┘ │  │ └───────────┘ │ │
│  └────────────────┘  └────────────────┘  └───────────────┘ │
│                              │                               │
│                              ▼                               │
│                   ┌─────────────────────┐                   │
│                   │    Share Scope      │                   │
│                   │  ┌───────────────┐  │                   │
│                   │  │ React 18.2.0  │  │                   │
│                   │  │ Lodash 4.17.0 │  │                   │
│                   │  │ Utils 1.0.0   │  │                   │
│                   │  └───────────────┘  │                   │
│                   └─────────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

### Key Architectural Elements

1. **Host Application**: The main application that consumes remote modules
2. **Remote Applications**: Applications that expose modules for others to use
3. **Remote Entry**: The entry point file that provides access to exposed modules
4. **Shared Modules**: Dependencies that can be shared between applications
5. **Module Federation Runtime**: The system that manages loading and sharing

## Module Federation in Micro-Frontend Architecture

### Traditional Micro-Frontend Challenges

Before Module Federation, micro-frontend architectures faced several challenges:

1. **Dependency Duplication**: Each micro-frontend bundled its own React, causing bloat
2. **Integration Complexity**: iframes or complex build-time integration
3. **Communication Overhead**: PostMessage or custom event systems
4. **Deployment Coordination**: Complex CI/CD pipelines

### How Module Federation Solves These

```javascript
// Shell Application (Container)
const ModuleFederationPlugin = require('@rspack/plugin-module-federation');

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'shell',
      remotes: {
        products: 'products@http://localhost:3001/remoteEntry.js',
        cart: 'cart@http://localhost:3002/remoteEntry.js',
        checkout: 'checkout@http://localhost:3003/remoteEntry.js',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        'react-router-dom': { singleton: true },
      },
    }),
  ],
};

// Products Micro-Frontend
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'products',
      filename: 'remoteEntry.js',
      exposes: {
        './ProductList': './src/components/ProductList',
        './ProductDetail': './src/components/ProductDetail',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
      },
    }),
  ],
};
```

### Benefits for Micro-Frontends

1. **True Runtime Integration**: No build-time coupling
2. **Shared Dependencies**: One React instance for all micro-frontends
3. **Independent Deployments**: Deploy without affecting others
4. **Type Safety**: Can maintain TypeScript interfaces
5. **Performance**: Lazy loading and optimal bundle sizes

## Getting Started

### Step 1: Install Dependencies

```bash
npm install --save-dev @rspack/core @rspack/cli @rspack/plugin-module-federation
```

### Step 2: Configure Your Host Application

```javascript
// rspack.config.js
const { ModuleFederationPlugin } = require('@rspack/plugin-module-federation');

module.exports = {
  entry: './src/index.js',
  mode: 'development',
  devServer: {
    port: 3000,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remote: 'remote@http://localhost:3001/remoteEntry.js',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
      },
    }),
  ],
};
```

### Step 3: Configure Your Remote Application

```javascript
// rspack.config.js
module.exports = {
  entry: './src/index.js',
  mode: 'development',
  devServer: {
    port: 3001,
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'remote',
      filename: 'remoteEntry.js',
      exposes: {
        './Button': './src/components/Button',
        './Card': './src/components/Card',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
      },
    }),
  ],
};
```

### Step 4: Use Remote Modules

```javascript
// In your host application
import React, { Suspense } from 'react';

const RemoteButton = React.lazy(() => import('remote/Button'));

function App() {
  return (
    <div>
      <h1>Host Application</h1>
      <Suspense fallback="Loading Button...">
        <RemoteButton onClick={() => alert('Clicked!')}>
          Click me!
        </RemoteButton>
      </Suspense>
    </div>
  );
}
```

## Common Use Cases

### 1. **Shared Component Library**
```javascript
// Design System Application
exposes: {
  './Button': './src/components/Button',
  './Input': './src/components/Input',
  './Card': './src/components/Card',
  './theme': './src/theme/index',
}
```

### 2. **Feature Modules**
```javascript
// E-commerce Features
exposes: {
  './ProductCatalog': './src/features/ProductCatalog',
  './ShoppingCart': './src/features/ShoppingCart',
  './UserProfile': './src/features/UserProfile',
}
```

### 3. **Utility Sharing**
```javascript
// Shared Utilities
shared: {
  './utils/api': './src/utils/api',
  './utils/auth': './src/utils/auth',
  './utils/validation': './src/utils/validation',
}
```

### 4. **Multi-Brand Applications**
```javascript
// Brand-specific configurations
remotes: {
  brandA: 'brandA@http://brandA.com/remoteEntry.js',
  brandB: 'brandB@http://brandB.com/remoteEntry.js',
}
```

## Best Practices

### 1. **Version Management**
Always specify version ranges for shared dependencies:
```javascript
shared: {
  react: {
    singleton: true,
    requiredVersion: '^18.0.0',
    strictVersion: false,
  },
}
```

### 2. **Error Boundaries**
Wrap remote components in error boundaries:
```javascript
<ErrorBoundary fallback={<ErrorUI />}>
  <RemoteComponent />
</ErrorBoundary>
```

### 3. **Loading States**
Always provide meaningful loading states:
```javascript
<Suspense fallback={<Skeleton />}>
  <RemoteModule />
</Suspense>
```

### 4. **Type Safety**
Create type definitions for remote modules:
```typescript
declare module 'remote/Button' {
  import { ButtonProps } from './types';
  const Button: React.FC<ButtonProps>;
  export default Button;
}
```

### 5. **Performance Optimization**
- Use eager loading for critical shared dependencies
- Implement proper code splitting
- Monitor bundle sizes
- Use CDN for remote entries in production

## Conclusion

Module Federation represents a paradigm shift in how we build and deploy web applications. By enabling true runtime module sharing, it solves many traditional challenges in micro-frontend architectures while providing the flexibility and independence that modern development teams need.

Whether you're building a large enterprise application with multiple teams or simply want to share code between projects more efficiently, Module Federation provides the tools and patterns to make it happen seamlessly.

In the next sections, we'll dive deeper into advanced configurations, optimization techniques, and real-world implementation patterns.