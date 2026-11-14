import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

function exclusionArrays(bytes) {
  return [
    ...bytes
      .toString('utf8')
      .matchAll(/(\[[^\n]*\])\.indexOf\(__rspack_import_key\) < 0/g),
  ].map((match) => JSON.parse(match[1]));
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
  description:
    'should preserve bundle bytes and full hash after persistent cache recovery of export-star exclusions',
  options(context) {
    const cacheDir = context.getDist('.cache');
    fs.rmSync(cacheDir, { recursive: true, force: true });
    const barrel = context.getSource('barrel.js');
    const barrelBuilds = [];
    context.setValue('barrelBuilds', barrelBuilds);
    return {
      mode: 'development',
      target: 'node',
      context: context.getSource(),
      entry: './index.js',
      devtool: false,
      incremental: false,
      cache: {
        type: 'persistent',
        buildDependencies: [import.meta.filename],
        storage: { type: 'filesystem', location: cacheDir },
      },
      optimization: {
        minimize: false,
        concatenateModules: false,
        inlineExports: false,
        usedExports: false,
        splitChunks: false,
        runtimeChunk: false,
      },
      output: {
        path: context.getDist('output'),
        filename: 'main.js',
        library: { type: 'commonjs2' },
      },
      plugins: [
        {
          apply(compiler) {
            const run = barrelBuilds.length;
            barrelBuilds.push(0);
            compiler.hooks.compilation.tap('ReexportOrder', (compilation) => {
              compilation.hooks.buildModule.tap('ReexportOrder', (module) => {
                if (module.resource === barrel) barrelBuilds[run]++;
              });
            });
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
    const builds = [];
    for (let run = 0; run < 2; run++) {
      if (run > 0) {
        await manager.close();
        manager.createCompiler().outputFileSystem = fs;
      }
      const stats = await manager.build();
      expect(stats.toJson({ all: false, errors: true }).errors).toEqual([]);
      builds.push({
        bytes: fs.readFileSync(context.getDist(path.join('output', 'main.js'))),
        hash: stats.compilation.fullHash,
      });
    }
    context.setValue('builds', builds);
  },
  check({ context }) {
    const [cold, warm] = context.getValue('builds');
    // buildModule is not called for a barrel restored from the filesystem cache.
    expect(context.getValue('barrelBuilds')).toEqual([1, 0]);
    expect(
      warm.bytes,
      `cold exclusions: ${JSON.stringify(exclusionArrays(cold.bytes))}; warm exclusions: ${JSON.stringify(exclusionArrays(warm.bytes))}; cold hash: ${cold.hash}; warm hash: ${warm.hash}`,
    ).toEqual(cold.bytes);
    expect(warm.hash).toBe(cold.hash);
    const expectedExclusions = [
      'default',
      'item00',
      'item01',
      'item02',
      'item03',
      'item04',
      'item05',
      'item06',
      'item07',
      'item08',
      'item09',
      'item10',
      'item11',
      'item12',
      'item13',
      'item14',
      'item15',
      'item16',
      'item17',
      'item18',
      'item19',
      'item20',
      'item21',
      'item22',
      'item23',
      'viaStar',
    ];
    for (const { bytes, hash } of [cold, warm]) {
      expect(exclusionArrays(bytes)).toEqual([expectedExclusions]);
      const module = { exports: {} };
      vm.runInNewContext(bytes.toString('utf8'), {
        module,
        exports: module.exports,
      });
      expect(module.exports.namespace.item05).toBe('item05');
      expect(module.exports.namespace.viaStar).toBe('known');
      expect(module.exports.namespace.dynamicOnly).toBe('dynamic');
      expect(Object.hasOwn(module.exports.namespace, 'default')).toBe(false);
      expect(module.exports.hash).toBe(hash);
    }
  },
};
