import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';

function orders(prefix = '', remaining = 'ABCD') {
  const result = prefix ? [prefix] : [];
  for (const name of remaining) result.push(...orders(prefix + name, remaining.replace(name, '')));
  return result;
}
async function execute(build, order) {
  const requests = new Set(), steps = [], values = {};
  let randomCalls = 0;
  const context = {setTimeout, clearTimeout, console, Math: Object.create(Math)};
  context.self = context;
  context.Math.random = () => ++randomCalls / 10;
  const sandbox = vm.createContext(context);
  const head = {appendChild(element) {
    const name = path.basename(element.src || element.href);
    requests.add(name); element.parentNode = head;
    if (name.endsWith('.js')) vm.runInContext(build.files[name], sandbox);
    element.onload?.({type: 'load', target: element});
  },removeChild() {}};
  context.document = {head, getElementsByTagName: () => [], querySelectorAll: () => [], createElement: () => ({attributes: {}, setAttribute(name,value) {this.attributes[name] = value;}, getAttribute(name) {return this.attributes[name];}, removeAttribute(name) {delete this.attributes[name];}})};
  vm.runInContext(build.files['main.js'], sandbox);
  requests.add('main.js');
  for (const name of order) {
    values[name] = (await context['load' + name]()).value;
    const copies = {};
    for (const chunk of build.chunks) {
      if (!chunk.files.some(file => requests.has(file))) continue;
      for (const module of chunk.modules) copies[module] = (copies[module] || 0) + 1;
    }
    steps.push(copies);
  }
  return {values, randomCalls, steps};
}
export default [false, true].map(usedExports => ({
  description: `residual support before SourceType size pruning, usedExports=${usedExports}`,
  options(context) {
    return {mode: 'production', entry: './index.js', cache: false, incremental: false, devtool: false,
      experiments: {css: true}, module: {rules: [{test: /\.css$/, type: 'css/auto'}]},
      output: {path: context.getDist(String(usedExports)), publicPath: '/', filename: '[name].js', chunkFilename: '[name].js'},
      optimization: {minimize: false, concatenateModules: false, moduleIds: 'named', chunkIds: 'named',
        splitChunks: {chunks: 'all', usedExports, dedupDepth: 1, minSize: 1, minSizeReduction: 80,
          cacheGroups: {default: {minChunks: 2, priority: -20, reuseExistingChunk: true}, defaultVendors: false}}
      }
    };
  },
  compiler(_context, compiler) {compiler.outputFileSystem = fs;},
  async build(_context, compiler) {
    const previous = process.env.RSPACK_RESIDUAL_REUSE;
    const run = async enabled => {
      process.env.RSPACK_RESIDUAL_REUSE = enabled ? '1' : '0';
      const stats = await new Promise((resolve,reject) => compiler.run((error,stats) => error ? reject(error) : resolve(stats)));
      assert.equal(stats.hasErrors(),false,stats.toString());
      assert.equal(compiler.options.optimization.splitChunks.minSize,1);
      assert.equal(compiler.options.optimization.splitChunks.minSizeReduction,80);
      const compilation = stats.compilation;
      const chunks = [...compilation.chunks].map(chunk => ({name: chunk.name, groups: [...chunk.groupsIterable].map(group=>group.name).sort(), files: [...chunk.files],
        modules: compilation.chunkGraph.getChunkModules(chunk).map(module => path.basename(module.nameForCondition?.() || ''))}));
      const files = Object.fromEntries(Object.keys(compilation.assets).filter(name=>/\.(js|css)$/.test(name)).map(name=>[name,fs.readFileSync(path.join(compiler.outputPath,name),'utf8')]));
      return {chunks, files};
    };
    try {
      const baseline = await run(false), changed = await run(true);
      assert.equal(baseline.chunks.filter(chunk=>chunk.modules.includes('m.js')).length,2);
      assert.equal(changed.chunks.filter(chunk=>chunk.modules.includes('m.js')).length,1);
      const shared = changed.chunks.find(chunk=>chunk.modules.includes('m.js'));
      assert.deepEqual(shared.groups,['A','B']);
      for (const order of orders()) {
        const a = await execute(baseline,order), b = await execute(changed,order);
        assert.deepEqual(b.values,a.values);
        assert.equal(b.randomCalls,a.randomCalls);
        for (let step=0;step<a.steps.length;step++) {
          for (const [module,count] of Object.entries(b.steps[step])) assert(count <= (a.steps[step][module] || 0),`${order} ${step} adds ${module}`);
        }
      }
      const repeated = await run(true);
      assert.deepEqual(repeated.files,changed.files);
    } finally {
      if (previous === undefined) delete process.env.RSPACK_RESIDUAL_REUSE;
      else process.env.RSPACK_RESIDUAL_REUSE = previous;
    }
  }
}));
