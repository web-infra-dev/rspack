const { CircularCheckRspackPlugin } = require('@rspack/core');

const PLUGIN_NAME = 'CircularCheckRspackPluginConfigCase';

function normalizeMessage(message) {
  return message.replaceAll('\\', '/');
}

function getMessages(stats, type) {
  return stats
    .toJson({
      all: false,
      errors: true,
      warnings: true,
    })
    [type].map((item) => normalizeMessage(item.message));
}

function expectNoDiagnostics(stats) {
  expect(getMessages(stats, 'warnings')).toHaveLength(0);
  expect(getMessages(stats, 'errors')).toHaveLength(0);
}

function expectCircularDiagnostic(stats, type, expectedPath) {
  const messages = getMessages(stats, type);
  const otherType = type === 'warnings' ? 'errors' : 'warnings';
  expect(messages).toHaveLength(1);
  expect(getMessages(stats, otherType)).toHaveLength(0);
  expect(messages[0]).toMatch(/Circular dependency detected/);
  expect(messages[0]).toContain(expectedPath);
}

class AssertStatsPlugin {
  constructor(assert) {
    this.assert = assert;
  }

  apply(compiler) {
    compiler.hooks.done.tap(PLUGIN_NAME, this.assert);
  }
}

function createCase(name, entry, plugins, assert, extra = {}) {
  return {
    name,
    mode: 'development',
    entry,
    plugins: [...plugins, new AssertStatsPlugin(assert)],
    ...extra,
  };
}

const efgCycle = './deps/e.js -> ./deps/f.js -> ./deps/g.js -> ./deps/e.js';

module.exports = [
  createCase(
    'detects-basic-cycle',
    './deps/a.js',
    [new CircularCheckRspackPlugin()],
    (stats) => {
      expectCircularDiagnostic(
        stats,
        'warnings',
        './deps/b.js -> ./deps/c.js -> ./deps/b.js',
      );
    },
  ),
  createCase(
    'detects-deep-cycle',
    './deps/d.js',
    [new CircularCheckRspackPlugin()],
    (stats) => {
      expectCircularDiagnostic(stats, 'warnings', efgCycle);
    },
  ),
  createCase(
    'fail-on-error',
    './deps/d.js',
    [
      new CircularCheckRspackPlugin({
        failOnError: true,
      }),
    ],
    (stats) => {
      expectCircularDiagnostic(stats, 'errors', efgCycle);
    },
  ),
  createCase(
    'exclude',
    './deps/d.js',
    [
      new CircularCheckRspackPlugin({
        exclude: /f\.js/,
      }),
    ],
    expectNoDiagnostics,
  ),
  createCase(
    'include',
    './deps/d.js',
    [
      new CircularCheckRspackPlugin({
        include: /f\.js/,
      }),
    ],
    (stats) => {
      expectCircularDiagnostic(stats, 'warnings', efgCycle);
    },
  ),
  createCase(
    'context-module',
    './deps/h.js',
    [new CircularCheckRspackPlugin()],
    expectNoDiagnostics,
  ),
  (() => {
    let detectedPaths;
    return createCase(
      'on-detected-overrides-default-report',
      './deps/d.js',
      [
        new CircularCheckRspackPlugin({
          onDetected({ paths }) {
            detectedPaths = paths;
          },
        }),
      ],
      (stats) => {
        expectNoDiagnostics(stats);
        expect(detectedPaths).toEqual([
          './deps/e.js',
          './deps/f.js',
          './deps/g.js',
          './deps/e.js',
        ]);
      },
    );
  })(),
  createCase(
    'on-detected-can-report',
    './deps/d.js',
    [
      new CircularCheckRspackPlugin({
        onDetected({ paths, compilation }) {
          compilation.warnings.push(new Error(paths.join(' -> ')));
        },
      }),
    ],
    (stats) => {
      const messages = getMessages(stats, 'warnings');
      expect(messages).toHaveLength(1);
      expect(messages[0]).toContain(efgCycle);
      expect(getMessages(stats, 'errors')).toHaveLength(0);
    },
  ),
  createCase(
    'module-concat-plugin-compat',
    './deps/module-concat-plugin-compat/index.js',
    [new CircularCheckRspackPlugin()],
    (stats) => {
      expectCircularDiagnostic(
        stats,
        'warnings',
        './deps/module-concat-plugin-compat/a.js -> ./deps/module-concat-plugin-compat/b.js -> ./deps/module-concat-plugin-compat/a.js',
      );
    },
    {
      optimization: {
        concatenateModules: true,
      },
    },
  ),
  createCase(
    'ignores-this-reference',
    './deps/self-referencing/uses-this.js',
    [new CircularCheckRspackPlugin()],
    expectNoDiagnostics,
  ),
  createCase(
    'ignores-module-exports-reference',
    './deps/self-referencing/uses-exports.js',
    [new CircularCheckRspackPlugin()],
    expectNoDiagnostics,
  ),
  createCase(
    'ignores-self-import',
    './deps/self-referencing/imports-self.js',
    [new CircularCheckRspackPlugin()],
    (stats) => {
      expectCircularDiagnostic(
        stats,
        'warnings',
        './deps/self-referencing/imports-self.js -> ./deps/self-referencing/imports-self.js',
      );
    },
  ),
  createCase(
    'detects-esm-self-import',
    './deps/self-referencing/esm-imports-self.js',
    [new CircularCheckRspackPlugin()],
    (stats) => {
      expectCircularDiagnostic(
        stats,
        'warnings',
        './deps/self-referencing/esm-imports-self.js -> ./deps/self-referencing/esm-imports-self.js',
      );
    },
  ),
  createCase(
    'works-with-typescript',
    './deps/ts/a.tsx',
    [new CircularCheckRspackPlugin()],
    expectNoDiagnostics,
    {
      resolve: {
        extensions: ['...', '.ts', '.tsx'],
      },
      module: {
        rules: [
          {
            test: /\.tsx?$/,
            loader: 'builtin:swc-loader',
          },
        ],
      },
    },
  ),
];
