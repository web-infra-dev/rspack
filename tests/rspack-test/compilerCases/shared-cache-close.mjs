import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { closeCompiler, runCompiler } from '@rspack/test-tools/helper/lifecycle';

const fixture = path.join(
  import.meta.dirname,
  '../multiCompilerCases/loader-cache',
);

function createOptions(context, name) {
  return {
    name,
    context: fixture,
    mode: 'development',
    entry: './index.js',
    output: { path: context.getDist(name) },
    cache: {
      type: 'persistent',
      // A hit must come from the shared memory cache.
      readonly: true,
      maxMemoryGenerations: Infinity,
      storage: { type: 'filesystem', location: context.getDist('cache') },
    },
    experiments: {
      newCache: {
        codeGeneration: false,
        devtool: false,
        loader: true,
        minimize: false,
        module: false,
      },
    },
    module: {
      rules: [
        {
          test: /index\.js$/,
          use: [
            {
              loader: path.join(fixture, 'count-loader.mjs'),
              options: { counterFile: context.getDist('loader-runs.txt') },
              cache: true,
            },
          ],
        },
      ],
    },
  };
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description: 'should keep shared storage alive until the last compiler closes',
  options(context) {
    fs.rmSync(context.getDist('cache'), { recursive: true, force: true });
    fs.rmSync(context.getDist('loader-runs.txt'), { force: true });
    return createOptions(context, 'first');
  },
  async build(context, first) {
    const second = rspack(createOptions(context, 'second'));
    const third = rspack(createOptions(context, 'third'));
    const loaderRuns = () =>
      fs.readFileSync(context.getDist('loader-runs.txt'), 'utf8');
    try {
      try {
        expect((await runCompiler(first)).hasErrors()).toBe(false);
        expect((await runCompiler(second)).hasErrors()).toBe(false);
        expect(loaderRuns()).toBe('x');
      } finally {
        await closeCompiler(first);
      }

      expect((await runCompiler(third)).hasErrors()).toBe(false);
      expect(loaderRuns()).toBe('x');
    } finally {
      await Promise.all([closeCompiler(third), closeCompiler(second)]);
    }

    const fourth = rspack(createOptions(context, 'fourth'));
    try {
      expect((await runCompiler(fourth)).hasErrors()).toBe(false);
      expect(loaderRuns()).toBe('xx');
    } finally {
      await closeCompiler(fourth);
    }
  },
};
