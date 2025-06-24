# Configuration Basics

## Essential Configuration Options

### Entry Points

```javascript
module.exports = {
  // Single entry
  entry: './src/index.js',
  
  // Multiple entries
  entry: {
    app: './src/app.js',
    vendor: './src/vendor.js',
  },
};
```

### Output Configuration

```javascript
module.exports = {
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].[contenthash].js',
    clean: true, // Clean output directory before build
  },
};
```

### Module Processing

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'swc-loader',
          options: {
            jsc: {
              parser: {
                syntax: 'ecmascript',
              },
            },
          },
        },
      },
      {
        test: /\.css$/,
        use: ['style-loader', 'css-loader'],
      },
    ],
  },
};
```

### Development vs Production

```javascript
module.exports = (env, argv) => {
  const isProduction = argv.mode === 'production';
  
  return {
    mode: isProduction ? 'production' : 'development',
    devtool: isProduction ? 'source-map' : 'eval-cheap-module-source-map',
    optimization: {
      minimize: isProduction,
    },
  };
};
```

## Common Configuration Patterns

### Code Splitting

```javascript
module.exports = {
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
      },
    },
  },
};
```

### Asset Management

```javascript
module.exports = {
  module: {
    rules: [
      {
        test: /\.(png|jpg|gif|svg)$/,
        type: 'asset/resource',
        generator: {
          filename: 'images/[name].[hash][ext]',
        },
      },
      {
        test: /\.(woff|woff2|eot|ttf|otf)$/,
        type: 'asset/resource',
        generator: {
          filename: 'fonts/[name].[hash][ext]',
        },
      },
    ],
  },
};
```

### Development Server

```javascript
module.exports = {
  devServer: {
    static: './dist',
    hot: true,
    port: 8080,
    open: true,
  },
};
```

## Next Steps

- [Common Patterns](common-patterns.md) - Explore advanced configuration patterns
- [Module Federation](../features/module-federation/overview.md) - Learn about micro-frontend architecture
- [Performance Guide](../performance/overview.md) - Optimize your builds