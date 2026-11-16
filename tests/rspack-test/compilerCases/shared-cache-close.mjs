import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';
import {
  closeCompiler,
  runCompiler,
} from '@rspack/test-tools/helper/lifecycle';

const fixture = path.join(
  import.meta.dirname,
  '../multiCompilerCases/loader-cache',
);

function createOptions(
  context,
  name,
  type,
  { readonly = false, maxMemoryGenerations = Infinity } = {},
) {
  return {
    name,
    context: fixture,
    mode: 'development',
    entry: './index.js',
    output: { path: context.getDist(name) },
    cache:
      type === 'memory'
        ? { type }
        : {
            type: 'persistent',
            readonly,
            maxMemoryGenerations,
            storage: {
              type: 'filesystem',
              directory: context.getDist('cache'),
            },
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

/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
const closeCases = ['persistent', 'memory'].map((type) => ({
  name: type,
  description: `should keep the shared ${type} loader cache alive until the last compiler closes`,
  options(context) {
    fs.rmSync(context.getDist('cache'), { recursive: true, force: true });
    fs.rmSync(context.getDist('loader-runs.txt'), { force: true });
    return createOptions(context, 'first', type);
  },
  async build(context, first) {
    const second = rspack(createOptions(context, 'second', type));
    const third = rspack(createOptions(context, 'third', type));
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

    const fourth = rspack(createOptions(context, 'fourth', type));
    try {
      expect((await runCompiler(fourth)).hasErrors()).toBe(false);
      expect(loaderRuns()).toBe(type === 'memory' ? 'xx' : 'x');
    } finally {
      await closeCompiler(fourth);
    }
  },
}));

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
const readonlyCase = {
  name: 'readonly-first',
  description: 'should enforce loader cache readonly per compiler',
  options(context) {
    fs.rmSync(context.getDist('cache'), { recursive: true, force: true });
    fs.rmSync(context.getDist('loader-runs.txt'), { force: true });
    return createOptions(context, 'reader', 'persistent', {
      readonly: true,
      maxMemoryGenerations: 0,
    });
  },
  async build(context, reader) {
    const createCompiler = (name, readonly) =>
      rspack(
        createOptions(context, name, 'persistent', {
          readonly,
          maxMemoryGenerations: 0,
        }),
      );

    async function build(compiler, expected) {
      const stats = await runCompiler(compiler);
      expect(stats.hasErrors()).toBe(false);
      const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
      expect(cacheInfo?.[`${compiler.compilerPath}|loader`]).toEqual(expected);
    }

    const writer = createCompiler('writer', false);
    try {
      await build(reader, { hits: 0, misses: 1, reused: 0, stores: 0 });
      await build(writer, { hits: 0, misses: 1, reused: 0, stores: 1 });
    } finally {
      await Promise.all([closeCompiler(reader), closeCompiler(writer)]);
    }

    const reopened = createCompiler('reopened', true);
    try {
      await build(reopened, { hits: 1, misses: 0, reused: 1, stores: 0 });
    } finally {
      await closeCompiler(reopened);
    }
  },
};

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
const sourceMapCase = {
  name: 'source-map',
  description: 'should only reuse loader results with matching source map requirements',
  options(context) {
    return createSourceMapOptions(context, 'first', false);
  },
  async build(context, first) {
    const cheap = rspack(
      createSourceMapOptions(context, 'cheap', 'cheap-source-map'),
    );
    const full = rspack(createSourceMapOptions(context, 'full', 'source-map'));
    const reused = rspack(createSourceMapOptions(context, 'reused', 'source-map'));
    try {
      for (const [compiler, hits] of [
        [first, 0],
        [cheap, 1],
        [full, 0],
        [reused, 1],
      ]) {
        const stats = await runCompiler(compiler);
        expect(stats.hasErrors()).toBe(false);
        const { cacheInfo } = stats.toJson({ all: false, cacheInfo: true });
        expect(cacheInfo?.[`${compiler.compilerPath}|loader`]).toEqual({
          hits,
          misses: 1 - hits,
          reused: hits,
          stores: 1 - hits,
        });
      }

      for (const compiler of [full, reused]) {
        const bundle = fs.readFileSync(
          path.join(compiler.outputPath, 'main.js'),
          'utf8',
        );
        expect(bundle).toContain('expect(true).toBe(true)');
      }
    } finally {
      await Promise.all([first, cheap, full, reused].map(closeCompiler));
    }
  },
};

function createSourceMapOptions(context, name, devtool) {
  return {
    ...createOptions(context, name, 'memory'),
    context: path.join(
      import.meta.dirname,
      '../configCases/source-map/custom-loader',
    ),
    devtool,
    module: {
      rules: [
        { test: /index\.js$/, use: [{ loader: './loader.mjs', cache: true }] },
      ],
    },
  };
}

export default [...closeCases, readonlyCase, sourceMapCase];
