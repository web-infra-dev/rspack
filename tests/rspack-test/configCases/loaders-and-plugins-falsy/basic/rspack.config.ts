import { defineConfig } from '@rspack/cli';
import { DefinePlugin, type Compiler } from '@rspack/core';

const nullValue = null;
const undefinedValue = undefined;
const falseValue = false;
const zeroValue = 0;
const emptyStringValue = '';

class FailPlugin {
  apply() {
    throw new Error('FailedPlugin');
  }
}

class TestChildCompilationPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.make.tapAsync(
      'TestChildCompilationFailurePlugin',
      (compilation, cb) => {
        const child = compilation.createChildCompiler('name', {}, [
          undefinedValue && new FailPlugin(),
          nullValue && new FailPlugin(),
          falseValue && new FailPlugin(),
          zeroValue && new FailPlugin(),
          emptyStringValue && new FailPlugin(),
        ]);

        child.runAsChild((error) => cb(error));
      },
    );
  }
}

export default defineConfig({
  // Will failed because we don't have unknown-loader
  module: {
    defaultRules: [
      nullValue && {
        test: /\.js$/,
        loader: 'unknown-loader',
      },
      '...',
    ],
    rules: [
      nullValue && {
        test: /\.js$/,
        loader: 'unknown-loader',
      },
      {
        test: /foo\.js$/,
        oneOf: [
          nullValue && {
            resourceQuery: /inline/,
            loader: 'unknown-loader',
          },
          {
            resourceQuery: /external/,
            type: 'asset/resource',
          },
        ],
      },
      {
        test: /bar\.js$/,
        use: [nullValue && 'unknown-loader'],
      },
      {
        test: /baz\.js$/,
        resourceQuery: /custom-use/,
        use: () => [
          nullValue && {
            loader: 'unknown-loader',
          },
        ],
      },
      {
        test: /other\.js$/,
        rules: [
          nullValue && {
            loader: 'unknown-loader',
          },
          {
            loader: './loader.mjs',
          },
        ],
      },
    ],
  },
  plugins: [
    new DefinePlugin({
      ONE: JSON.stringify('ONE'),
    }),
    new TestChildCompilationPlugin(),
    undefinedValue && new FailPlugin(),
    nullValue && new FailPlugin(),
    falseValue && new FailPlugin(),
    zeroValue && new FailPlugin(),
    emptyStringValue && new FailPlugin(),
  ],
  optimization: {
    minimize: true,
    minimizer: [nullValue && new FailPlugin()],
  },
});
