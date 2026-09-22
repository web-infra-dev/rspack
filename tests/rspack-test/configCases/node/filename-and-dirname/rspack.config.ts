import { defineConfig } from '@rspack/cli';
import {
  type Configuration,
  DefinePlugin,
  type NodeOptions,
} from '@rspack/core';

const values: NodeOptions['__filename'][] = [
  true,
  'warn-mock',
  'mock',
  'node-module',
  'eval-only',
];

const config: Configuration[] = [];

// CommonJS
config.push(
  ...values
    .filter((item) => item !== 'node-module')
    .map<Configuration>((value) => ({
      target: 'node',
      node: {
        __filename: value,
        __dirname: value,
      },
      plugins: [
        new DefinePlugin({
          NODE_VALUE:
            typeof value === 'boolean' ? value : JSON.stringify(value),
          FORMAT: JSON.stringify('cjs'),
        }),
      ],
    })),
);

// ES modules
config.push(
  ...values.map<Configuration>((value) => ({
    target: 'node',
    node: {
      __filename: value,
      __dirname: value,
    },
    output: {
      module: true,
    },
    plugins: [
      new DefinePlugin({
        NODE_VALUE: typeof value === 'boolean' ? value : JSON.stringify(value),
        FORMAT: JSON.stringify('esm'),
      }),
    ],
  })),
);

config.push(
  ...values.map<Configuration>((value) => ({
    target: 'node',
    devtool: 'eval',
    node: {
      __filename: value,
      __dirname: value,
    },
    output: {
      module: true,
    },
    plugins: [
      new DefinePlugin({
        NODE_VALUE: typeof value === 'boolean' ? value : JSON.stringify(value),
        FORMAT: JSON.stringify('esm'),
      }),
    ],
  })),
);

// ES modules with support `import.meta.dirname` and `import.meta.filename`
config.push(
  ...values.map<Configuration>((value) => ({
    target: 'node',
    node: {
      __filename: value,
      __dirname: value,
    },
    output: {
      module: true,
      environment: {
        importMetaDirnameAndFilename: true,
      },
    },
    plugins: [
      new DefinePlugin({
        NODE_VALUE: typeof value === 'boolean' ? value : JSON.stringify(value),
        FORMAT: JSON.stringify('esm'),
      }),
    ],
  })),
);

config.push(
  ...values.map<Configuration>((value) => ({
    target: 'node',
    devtool: 'eval',
    node: {
      __filename: value,
      __dirname: value,
    },
    output: {
      module: true,
      environment: {
        importMetaDirnameAndFilename: true,
      },
    },
    plugins: [
      new DefinePlugin({
        NODE_VALUE: typeof value === 'boolean' ? value : JSON.stringify(value),
        FORMAT: JSON.stringify('esm'),
      }),
    ],
  })),
);

config.push({
  entry: './cjs-false.js',
  target: 'node',
  node: {
    __filename: false,
    __dirname: false,
  },
});

config.push({
  entry: './esm-false.js',
  target: 'node',
  node: {
    __filename: false,
    __dirname: false,
  },
  output: {
    module: true,
  },
});

config.push({
  entry: './esm-false.js',
  target: 'node',
  output: {
    module: true,
  },
  module: {
    parser: {
      javascript: {
        importMeta: false,
      },
    },
  },
});

config.push({
  entry: './esm-false.js',
  target: 'node',
  node: false,
  output: {
    module: true,
  },
});

config.push({
  entry: './esm-false.js',
  target: 'node',
  module: {
    parser: {
      javascript: {
        node: false,
      },
    },
  },
  output: {
    module: true,
  },
});

export default defineConfig(config);
