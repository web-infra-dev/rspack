const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

let builtModules = [];
const serializationWarnings = [];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    'should skip non-serializable modules without blocking other disk cache entries',
  options(context) {
    serializationWarnings.length = 0;
    const root = context.getDist();
    fs.rmSync(root, { recursive: true, force: true });
    return {
      context: path.resolve(
        __dirname,
        '../fixtures/module-cache-serialization',
      ),
      mode: 'development',
      entry: './index.js',
      target: 'node',
      devtool: false,
      incremental: false,
      cache: {
        type: 'persistent',
        buildDependencies: [__filename],
        storage: {
          type: 'filesystem',
          location: path.join(root, '.cache'),
        },
      },
      experiments: {
        newCache: {
          module: true,
          loader: false,
          codeGeneration: false,
          devtool: false,
          minimize: false,
        },
      },
      output: {
        path: root,
        filename: 'main.js',
        library: { type: 'commonjs2' },
      },
      module: {
        rules: [
          {
            test: /unsupported\.js$/,
            loader: 'builtin:test-non-serializable-module-loader',
          },
        ],
      },
      plugins: [
        {
          apply(compiler) {
            compiler.hooks.compilation.tap(
              'ModuleCacheSerialization',
              (compilation) => {
                compilation.hooks.buildModule.tap(
                  'ModuleCacheSerialization',
                  (module) => {
                    if (module.resource)
                      builtModules.push(path.basename(module.resource));
                  },
                );
              },
            );
            compiler.hooks.infrastructureLog.tap(
              'ModuleCacheSerialization',
              (name, type, args) => {
                if (name === 'rspack.cache.IdleFileCache' && type === 'warn') {
                  serializationWarnings.push(args.join(' '));
                  return true;
                }
              },
            );
          },
        },
      ],
    };
  },
  compiler(_, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const manager = context.getCompiler();
    const build = async (expected) => {
      builtModules = [];
      const stats = await manager.build();
      expect(stats.hasErrors()).toBe(false);
      expect(builtModules.sort()).toEqual(expected);
      const module = { exports: {} };
      vm.runInNewContext(fs.readFileSync(context.getDist('main.js'), 'utf-8'), {
        module,
      });
      expect(module.exports).toEqual(['stable', 'unsupported']);
    };

    await build(['index.js', 'stable.js', 'unsupported.js']);
    // Closing flushes the batch containing both supported and unsupported entries.
    await manager.close();
    // A module-level preflight would discard this entry before the file cache
    // could report its serialization failure.
    expect(serializationWarnings).toHaveLength(1);
    expect(serializationWarnings[0]).toContain(
      'Skipped non-serializable cache entry',
    );
    expect(serializationWarnings[0]).toContain('unsupported.js');
    expect(serializationWarnings[0]).toContain('unsupported field');

    manager.createCompiler();
    await build(['unsupported.js']);
  },
};
