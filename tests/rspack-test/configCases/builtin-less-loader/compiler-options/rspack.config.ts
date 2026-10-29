import path from 'node:path';
import { defineConfig } from '@rspack/cli';
import type { LessLoaderOptions } from '@rspack/core';

const cases: Record<string, LessLoaderOptions> = {
  defaults: {},
  'relative-urls-disabled': { lessOptions: { relativeUrls: false } },
  'rewrite-urls-overridden': { lessOptions: { rewriteUrls: 'off' } },
  'rewrite-urls-off': {
    lessOptions: { relativeUrls: false, rewriteUrls: 'off' },
  },
  'rewrite-urls-local': {
    lessOptions: { relativeUrls: false, rewriteUrls: 1 },
  },
  'numeric-math': { lessOptions: { math: 0 } },
  'strict-math': { lessOptions: { strictMath: true, math: 'always' } },
  'line-numbers': { lessOptions: { dumpLineNumbers: 'comments' } },
  banner: { lessOptions: { banner: '.banner { color: red; }\n' } },
  'skip-imports': { lessOptions: { processImports: false } },
  paths: { lessOptions: { paths: path.join(import.meta.dirname, 'nested') } },
  filename: {
    lessOptions: {
      filename: path.join(import.meta.dirname, 'renamed.less'),
      dumpLineNumbers: 'comments',
    },
  },
  'compiler-map': {
    sourceMap: false,
    lessOptions: {
      sourceMap: {
        sourceMapBasepath: '',
        sourceMapRootpath: '/sources',
        sourceMapOutputFilename: 'custom.css',
        sourceMapURL: 'custom.css.map',
        outputSourceFiles: true,
      },
    },
  },
  'loader-map': {
    sourceMap: true,
    lessOptions: {
      sourceMap: {
        sourceMapFileInline: true,
        sourceMapRootpath: '/ignored',
        outputSourceFiles: false,
      },
    },
  },
  'loader-map-overrides-false': {
    sourceMap: true,
    lessOptions: { sourceMap: false },
  },
};

export default defineConfig({
  devtool: false,
  module: {
    rules: Object.entries(cases).map(([name, options]) => ({
      test: /\.less$/,
      resourceQuery: new RegExp(`^\\?${name}(&reference)?$`),
      type: 'asset/source',
      oneOf: [
        {
          resourceQuery: /&reference$/,
          use: [
            './capture-loader.mjs',
            {
              loader: 'less-loader',
              options: { ...options, webpackImporter: false },
            },
          ],
        },
        {
          use: [
            './capture-loader.mjs',
            { loader: 'builtin:less-loader', options },
          ],
        },
      ],
    })),
  },
});
