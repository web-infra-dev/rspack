# Module Federation Documentation

## Overview

Module Federation is a powerful feature that allows multiple independent builds to form a single application. It enables runtime code sharing between different JavaScript applications, making it possible to dynamically load code from other builds and share dependencies efficiently.

### Key Benefits
- **Dynamic Module Loading**: Load modules from remote applications at runtime
- **Dependency Sharing**: Share common dependencies across applications to reduce bundle size
- **Independent Deployments**: Deploy applications independently while maintaining integration
- **Version Management**: Handle multiple versions of shared dependencies gracefully

## Quick Start

### Basic Setup

1. **Configure a Host Application**
```javascript
// rspack.config.js
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remote: 'remote@http://localhost:3001/remoteEntry.js'
      },
      shared: ['react', 'react-dom']
    })
  ]
};
```

2. **Configure a Remote Application**
```javascript
// rspack.config.js
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'remote',
      filename: 'remoteEntry.js',
      exposes: {
        './Button': './src/Button'
      },
      shared: ['react', 'react-dom']
    })
  ]
};
```

3. **Use Remote Modules**
```javascript
// In your host application
const RemoteButton = React.lazy(() => import('remote/Button'));

function App() {
  return (
    <React.Suspense fallback="Loading...">
      <RemoteButton />
    </React.Suspense>
  );
}
```

## Documentation Structure

### Core Concepts

#### [Provider-Consumer Architecture](./provider-consumer-architecture.md)
Understand the fundamental relationship between applications that provide modules (remotes) and those that consume them (hosts). Learn about configuration patterns and best practices.

#### [Module Interlinking Architecture](./module-interlinking-architecture.md)
Deep dive into how modules are connected across applications, including the module resolution process and dependency graph management.

### Implementation Details

#### [Runtime Code Generation Analysis](./runtime-code-generation-analysis.md)
Explore how Module Federation generates runtime code, including the container creation process and dynamic module loading mechanisms.

#### [Consume Shared Analysis](./consume-shared-analysis.md)
Detailed analysis of the shared module consumption system, including version resolution, singleton management, and fallback strategies.

#### [Complete Sharing System Analysis](./complete-sharing-system-analysis.md)
Comprehensive overview of the entire sharing system, from configuration to runtime behavior, including advanced sharing strategies.

### CommonJS Integration

#### [CommonJS Integration with Module Federation](./commonjs-integration.md)
Comprehensive guide for integrating CommonJS modules with Module Federation, including configuration patterns, runtime behavior, and best practices for mixed module systems.

For detailed CommonJS implementation information, see the [CommonJS Documentation](../commonjs/README.md).

## Common Use Cases

### Micro-Frontend Architecture
Split large applications into smaller, manageable pieces that can be developed and deployed independently.

### Component Libraries
Share UI components across multiple applications without publishing to npm.

### A/B Testing
Dynamically load different versions of features for experimentation.

### Legacy Migration
Gradually migrate legacy applications by exposing new features as federated modules.

## Best Practices

1. **Version Management**: Always specify version ranges for shared dependencies
2. **Error Boundaries**: Implement error boundaries around federated components
3. **Loading States**: Provide meaningful loading indicators for remote modules
4. **Fallbacks**: Include fallback components for network failures
5. **Type Safety**: Use TypeScript declarations for remote modules

## Troubleshooting

### Common Issues

- **CORS Errors**: Ensure remote applications have proper CORS headers
- **Shared Dependency Conflicts**: Check version compatibility in shared configuration
- **Loading Failures**: Verify remote URLs and network connectivity
- **Build Errors**: Ensure all exposed modules are properly exported

### Debug Tools

- Browser DevTools Network tab for remote loading
- Module Federation DevTools extension
- Runtime logging with `process.env.NODE_ENV === 'development'`

## Further Reading

- [Webpack Module Federation Documentation](https://webpack.js.org/concepts/module-federation/)
- [Module Federation Examples Repository](https://github.com/module-federation/module-federation-examples)
- [Advanced Configuration Guide](https://module-federation.github.io/guide/)

## Contributing

To contribute to this documentation:
1. Follow the existing documentation structure
2. Include practical examples
3. Keep explanations clear and concise
4. Update this README when adding new documentation files

---

For questions or issues, please refer to the main Rspack documentation or create an issue in the repository.