const fs = require('node:fs');
const path = require('node:path');

const CASE_DIR = 'persistent-cache-dynamic-import-inner-graph';
const CACHE_DIR = '.cache';
const OUTPUT_DIR = 'output';
const WORK_DIR = 'workdir';

function readAssets(context) {
  return fs.readdirSync(context.getDist(OUTPUT_DIR)).sort();
}

function hasEagerFeature(context) {
  return fs
    .readFileSync(path.join(context.getDist(OUTPUT_DIR), 'main.js'), 'utf-8')
    .includes('PERSISTENT_CACHE_EAGER_FEATURE_MARKER');
}

async function recreateCompiler(context) {
  const compilerManager = context.getCompiler();
  await compilerManager.close();
  const compiler = compilerManager.createCompiler();
  compiler.outputFileSystem = fs;
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description:
    'should restore dynamic import inner graph conditions from persistent cache',
  options(context) {
    const sourceDir = path.resolve(__dirname, '../fixtures', CASE_DIR);
    const workDir = context.getDist(WORK_DIR);
    fs.rmSync(workDir, { recursive: true, force: true });
    fs.cpSync(sourceDir, workDir, { recursive: true });

    return {
      mode: 'production',
      target: 'node',
      context: workDir,
      entry: './index.js',
      experiments: {
        cache: {
          type: 'persistent',
          buildDependencies: [__filename],
          storage: {
            type: 'filesystem',
            location: context.getDist(CACHE_DIR),
          },
        },
      },
      optimization: {
        concatenateModules: false,
        innerGraph: true,
        minimize: false,
        providedExports: true,
        sideEffects: true,
        usedExports: true,
      },
      output: {
        path: context.getDist(OUTPUT_DIR),
        filename: 'main.js',
        chunkFilename: '[name].js',
        clean: true,
      },
    };
  },
  async compiler(_, compiler) {
    compiler.outputFileSystem = fs;
  },
  async build(context) {
    const compilerManager = context.getCompiler();
    const workDir = context.getDist(WORK_DIR);

    await compilerManager.build();
    context.setValue('unusedAssets', readAssets(context));
    context.setValue('unusedHasEagerFeature', hasEagerFeature(context));

    fs.copyFileSync(path.join(workDir, 'index.used.js'), path.join(workDir, 'index.js'));
    await recreateCompiler(context);
    await compilerManager.build();
    context.setValue('usedAssets', readAssets(context));
    context.setValue('usedHasEagerFeature', hasEagerFeature(context));

    fs.copyFileSync(path.join(workDir, 'index.unused.js'), path.join(workDir, 'index.js'));
    await recreateCompiler(context);
    await compilerManager.build();
    context.setValue('unusedAgainAssets', readAssets(context));
    context.setValue('unusedAgainHasEagerFeature', hasEagerFeature(context));
  },
  async check({ context }) {
    expect(context.getValue('unusedAssets')).toEqual(['main.js']);
    expect(context.getValue('unusedHasEagerFeature')).toBe(false);
    expect(context.getValue('usedAssets')).toEqual(['feature.js', 'main.js']);
    expect(context.getValue('usedHasEagerFeature')).toBe(true);
    expect(context.getValue('unusedAgainAssets')).toEqual(['main.js']);
    expect(context.getValue('unusedAgainHasEagerFeature')).toBe(false);
  },
};
