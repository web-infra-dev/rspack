import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  context: import.meta.dirname,
  module: {
    rules: [
      {
        test: /mixed-all-modes-[12]\.js$/,
        use: [
          {
            loader: './stage-loader.mjs',
            options: { name: 'main-0', emitResult: true },
          },
          {
            loader: './nopitch-loader.mjs',
            parallel: true,
            options: { name: 'parallel-nopitch-1' },
          },
          {
            loader: './nopitch-loader.mjs',
            parallel: true,
            options: { name: 'parallel-nopitch-2' },
          },
          {
            loader: './stage-loader.mjs',
            parallel: 'strict',
            options: { name: 'strict-3', expectStrict: true },
          },
          {
            loader: './stage-loader.mjs',
            parallel: 'strict',
            options: { name: 'strict-4', expectStrict: true },
          },
          {
            loader: './stage-loader.mjs',
            parallel: true,
            options: { name: 'parallel-5' },
          },
          {
            loader: './stage-loader.mjs',
            parallel: false,
            options: { name: 'main-6' },
          },
        ],
      },
      {
        test: /pitch-shortcircuit\.js$/,
        use: [
          {
            loader: './stage-loader.mjs',
            options: { name: 'main-0', emitResult: true },
          },
          {
            loader: './nopitch-loader.mjs',
            parallel: true,
            options: { name: 'parallel-nopitch-1' },
          },
          {
            loader: './stage-loader.mjs',
            parallel: 'strict',
            options: {
              name: 'strict-2',
              expectStrict: true,
              pitchReturn: true,
            },
          },
          {
            loader: './stage-loader.mjs',
            parallel: true,
            options: { name: 'parallel-unreached-3' },
          },
          {
            loader: './stage-loader.mjs',
            options: { name: 'main-unreached-4' },
          },
        ],
      },
      {
        test: path.join(import.meta.dirname, 'passthrough-additional-data.js'),
        use: [
          {
            loader: './parallel-consumer.mjs',
            parallel: true,
            options: {},
          },
          'builtin:test-passthrough-loader',
          {
            loader: './main-producer.mjs',
            parallel: false,
            options: {},
          },
        ],
      },
      {
        test: path.join(import.meta.dirname, 'pitch-additional-data.js'),
        use: [
          {
            loader: './parallel-consumer.mjs',
            parallel: true,
            options: {},
          },
          {
            loader: './main-pitch-producer.mjs',
            options: {},
          },
          {
            loader: './stage-loader.mjs',
            parallel: 'strict',
            options: { name: 'strict-unreached', expectStrict: true },
          },
        ],
      },
    ],
  },
});
