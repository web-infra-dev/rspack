# Common Patterns

## Library Development

### Creating a Library

```javascript
module.exports = {
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'my-library.js',
    library: {
      name: 'MyLibrary',
      type: 'umd',
    },
    globalObject: 'this',
  },
  externals: {
    lodash: {
      commonjs: 'lodash',
      commonjs2: 'lodash',
      amd: 'lodash',
      root: '_',
    },
  },
};
```

### Multiple Output Formats

```javascript
const configs = [
  // UMD build
  {
    entry: './src/index.js',
    output: {
      filename: 'library.umd.js',
      library: { type: 'umd', name: 'MyLibrary' },
    },
  },
  // ESM build
  {
    entry: './src/index.js',
    output: {
      filename: 'library.esm.js',
      library: { type: 'module' },
    },
    experiments: {
      outputModule: true,
    },
  },
];

module.exports = configs;
```

## Micro-frontend Architecture

### Module Federation Host

```javascript
const ModuleFederationPlugin = require('@module-federation/webpack');

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        mfe1: 'mfe1@http://localhost:3001/remoteEntry.js',
        mfe2: 'mfe2@http://localhost:3002/remoteEntry.js',
      },
    }),
  ],
};
```

### Module Federation Remote

```javascript
const ModuleFederationPlugin = require('@module-federation/webpack');

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'mfe1',
      filename: 'remoteEntry.js',
      exposes: {
        './Component': './src/Component.js',
        './utils': './src/utils.js',
      },
    }),
  ],
};
```

## Environment-Specific Builds

### Multi-Environment Configuration

```javascript
const configs = {
  development: {
    mode: 'development',
    devtool: 'eval-cheap-module-source-map',
    devServer: {
      hot: true,
      port: 3000,
    },
  },
  
  production: {
    mode: 'production',
    devtool: 'source-map',
    optimization: {
      minimize: true,
      sideEffects: false,
    },
  },
  
  test: {
    mode: 'development',
    devtool: false,
    externals: {
      'react/lib/ExecutionEnvironment': true,
      'react/lib/ReactContext': true,
    },
  },
};

module.exports = (env) => {
  return {
    ...configs[env.NODE_ENV || 'development'],
    // Common configuration
    entry: './src/index.js',
    output: {
      path: path.resolve(__dirname, 'dist'),
      filename: '[name].[contenthash].js',
    },
  };
};
```

## Advanced Code Splitting

### Route-based Splitting

```javascript
module.exports = {
  entry: {
    main: './src/index.js',
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        // Vendor chunks
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
        
        // Common chunks
        common: {
          minChunks: 2,
          name: 'common',
          chunks: 'all',
          enforce: true,
        },
        
        // Route-specific chunks
        routes: {
          test: /[\\/]src[\\/]routes[\\/]/,
          name: 'routes',
          chunks: 'all',
        },
      },
    },
  },
};
```

### Dynamic Imports

```javascript
// In your application code
async function loadComponent() {
  const { Component } = await import('./Component.js');
  return Component;
}

// Route-based lazy loading
const LazyComponent = React.lazy(() => import('./LazyComponent.js'));
```

## Asset Optimization

### Image Optimization

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.(png|jpg|jpeg|gif|svg)$/,
        type: 'asset',
        parser: {
          dataUrlCondition: {
            maxSize: 8192, // 8kb
          },
        },
        generator: {
          filename: 'images/[name].[hash:8][ext]',
        },
      },
    ],
  },
};
```

### CSS Optimization

```javascript
const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin');

module.exports = {
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [
          MiniCssExtractPlugin.loader,
          'css-loader',
          'postcss-loader',
        ],
      },
    ],
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: 'styles/[name].[contenthash].css',
    }),
  ],
  optimization: {
    minimizer: [
      '...',
      new CssMinimizerPlugin(),
    ],
  },
};
```

## Performance Optimization

### Bundle Analysis

```javascript
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

module.exports = {
  plugins: [
    process.env.ANALYZE && new BundleAnalyzerPlugin(),
  ].filter(Boolean),
};
```

### Caching Strategy

```javascript
module.exports = {
  output: {
    filename: '[name].[contenthash].js',
    chunkFilename: '[name].[contenthash].chunk.js',
  },
  optimization: {
    moduleIds: 'deterministic',
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
      },
    },
  },
  cache: {
    type: 'filesystem',
    buildDependencies: {
      config: [__filename],
    },
  },
};
```

## Testing Configuration

### Jest Integration

```javascript
// jest.config.js
module.exports = {
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  transform: {
    '^.+\\.(js|jsx|ts|tsx)$': ['@swc-node/jest'],
  },
  moduleFileExtensions: ['js', 'jsx', 'ts', 'tsx'],
  testEnvironment: 'jsdom',
};
```

## Next Steps

- [Architecture Overview](../architecture/overview.md) - Understand Rspack's internal architecture
- [Module Federation](../features/module-federation/overview.md) - Deep dive into micro-frontend patterns
- [Performance Guide](../performance/overview.md) - Advanced optimization techniques