import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    entry: './index.js',
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'development',
    devtool: false,
    node: {
      __dirname: false,
      __filename: false,
    },
    output: {
      filename: 'bundle0.js',
      cssFilename: 'bundle0.css',
      publicPath: '',
      assetModuleFilename: '[name]-[contenthash:6][ext]',
    },
    module: {
      rules: [
        {
          test: /default-stylesheet\.css$/,
          type: 'css',
          parser: {
            exportType: 'text',
          },
        },
        {
          test: /explicit-stylesheet\.css$/,
          type: 'css',
          parser: {
            as: 'stylesheet',
            exportType: 'text',
          },
        },
        {
          test: /(?:bare-block|no-trailing-semi|no-url|image-set|quoted-unquoted-urls|custom-props-and-strings|empty|stray-brace-semi|stray-brace-semi-semi|stray-brace-no-semi|unclosed-func|unclosed-string)\.css$/,
          type: 'css',
          parser: {
            as: 'block-contents',
            exportType: 'text',
          },
        },
        {
          test: /link-block\.css$/,
          type: 'css',
          parser: {
            as: 'block-contents',
          },
        },
        {
          test: /global-block\.css$/,
          type: 'css/global',
          parser: {
            as: 'block-contents',
            exportType: 'text',
          },
        },
        {
          test: /(?:nesting-and-vars|keyframes-eof\.module|global-then-local\.module)\.css$/,
          type: 'css/module',
          parser: {
            as: 'block-contents',
            exportType: 'text',
          },
        },
        {
          test: /pure-block\.module\.css$/,
          type: 'css/module',
          parser: {
            as: 'block-contents',
            pure: true,
            exportType: 'text',
          },
        },
      ],
    },
  },
  {
    entry: './global-index.js',
    externals: {
      fs: 'node-commonjs fs',
      path: 'node-commonjs path',
    },
    target: 'web',
    mode: 'development',
    devtool: false,
    output: {
      filename: 'bundle1.js',
      publicPath: '',
      assetModuleFilename: '[name]-[contenthash:6][ext]',
    },
    module: {
      parser: {
        css: {
          as: 'block-contents',
          exportType: 'text',
        },
        'css/global': {
          as: 'stylesheet',
          exportType: 'text',
        },
        'css/module': {
          as: 'block-contents',
          exportType: 'text',
        },
      },
      rules: [
        {
          test: /global-plain\.css$/,
          type: 'css',
        },
        {
          test: /global-auto\.css$/,
          type: 'css/auto',
        },
        {
          test: /global-css-global\.css$/,
          type: 'css/global',
        },
        {
          test: /global-css-module\.module\.css$/,
          type: 'css/module',
        },
        {
          test: /override-stylesheet\.css$/,
          type: 'css/auto',
          parser: {
            as: 'stylesheet',
            exportType: 'text',
          },
        },
      ],
    },
  },
]);
