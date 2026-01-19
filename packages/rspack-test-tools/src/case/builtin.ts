import path from 'node:path';
import type { RspackOptions } from '@rspack/core';
import fs from 'fs-extra';
import { merge } from 'webpack-merge';
import { isJavaScript } from '../helper';
import { BasicCaseCreator } from '../test/creator';
import type { ITestContext, ITestEnv } from '../type';
import { build, checkSnapshot, compiler } from './common';

const creator = new BasicCaseCreator({
  clean: true,
  describe: false,
  description(name) {
    return `${name} should match snapshot`;
  },
  steps: ({ name, src }) => {
    const cat = path.basename(path.dirname(src));
    const filter = FILTERS[cat];
    return [
      {
        config: async (context: ITestContext) => {
          const compiler = context.getCompiler();
          compiler.setOptions(defaultOptions(context));
        },
        compiler: async (context: ITestContext) => {
          await compiler(context, name);
        },
        build: async (context: ITestContext) => {
          await build(context, name);
        },
        run: async (env: ITestEnv, context: ITestContext) => {
          // no need to run, just check snapshot
        },
        check: async (env: ITestEnv, context: ITestContext) => {
          await checkSnapshot(env, context, name, 'output.snap.txt', filter);
        },
      },
    ];
  },
  concurrent: true,
});

export function createBuiltinCase(name: string, src: string, dist: string) {
  creator.create(name, src, dist);
}

export function defaultOptions(context: ITestContext): RspackOptions {
  let defaultOptions = {
    entry: {
      main: {
        import: './index',
      },
    },
    output: {
      publicPath: '/',
      path: context.getDist(),
      filename: '[name].js',
      chunkFilename: '[name].js',
      chunkFormat: 'array-push',
      cssFilename: '[name].css',
      cssChunkFilename: '[name].css',
      assetModuleFilename: '[hash][ext][query]',
      sourceMapFilename: '[file].map',
      chunkLoadingGlobal: 'rspackChunk',
      chunkLoading: 'jsonp',
      uniqueName: '__rspack_test__',
      enabledLibraryTypes: ['system'],
      strictModuleErrorHandling: false,
      iife: true,
      module: false,
      asyncChunks: true,
      scriptType: false,
      globalObject: 'self',
      importFunctionName: 'import',
      wasmLoading: 'fetch',
      webassemblyModuleFilename: '[hash].module.wasm',
      workerChunkLoading: 'import-scripts',
      workerWasmLoading: 'fetch',
      bundlerInfo: {
        force: false,
      },
    },
    module: {
      rules: [
        {
          test: /\.json$/,
          type: 'json',
        },
        {
          test: /\.mjs$/,
          type: 'javascript/esm',
        },
        {
          test: /\.cjs$/,
          type: 'javascript/dynamic',
        },
        {
          test: /\.js$/,
          type: 'javascript/auto',
        },
        {
          test: /\.css$/,
          type: 'css',
        },
        {
          test: /\.wasm$/,
          type: 'webassembly/async',
        },
      ],
    },
    node: {
      __dirname: 'mock',
      __filename: 'mock',
      global: 'warn',
    },
    optimization: {
      runtimeChunk: {
        name: 'runtime',
      },
      minimize: false,
      removeAvailableModules: true,
      removeEmptyChunks: true,
      moduleIds: 'named',
      chunkIds: 'named',
      sideEffects: false,
      mangleExports: false,
      inlineExports: false,
      usedExports: false,
      concatenateModules: false,
      nodeEnv: false,
    },
    resolve: {
      extensions: [
        '.js',
        '.jsx',
        '.ts',
        '.tsx',
        '.json',
        '.d.ts',
        '.css',
        '.wasm',
      ],
    },
    resolveLoader: {
      extensions: ['.js'],
    },
    experiments: {
      css: true,
      futureDefaults: true,
    },
    devtool: false,
    context: context.getSource(),
    plugins: [],
  } as RspackOptions;

  const testConfigFile = context.getSource('rspack.config.js');
  if (fs.existsSync(testConfigFile)) {
    const caseOptions = require(testConfigFile);
    if (caseOptions.entry) {
      delete defaultOptions.entry;
    }
    defaultOptions = merge(defaultOptions, caseOptions);
  }

  if (!global.printLogger) {
    defaultOptions.infrastructureLogging = {
      level: 'error',
    };
  }

  return defaultOptions;
}

const FILTERS: Record<string, (file: string) => boolean> = {
  'plugin-css': (file: string) => file.endsWith('.css'),
  'plugin-css-modules': (file: string) =>
    file.endsWith('.css') || (isJavaScript(file) && !file.includes('runtime')),
  'plugin-html': (file: string) => file.endsWith('.html'),
};
