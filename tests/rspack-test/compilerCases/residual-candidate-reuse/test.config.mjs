import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';

const variants = [
  {label: 'anonymous-depth0', depth: 0},
  {label: 'anonymous-depth1', depth: 1},
  {label: 'anonymous-depth3', depth: 3},
  {label: 'fixed-name', depth: 1, name: 'T'},
  {label: 'callback-name', depth: 1, name: () => 'T'},
  {label: 'priority-order', depth: 1, priority: true},
];
function orders(prefix = '', remaining = 'ABCDE') {
  const result = prefix ? [prefix] : [];
  for (const name of remaining) result.push(...orders(prefix + name, remaining.replace(name, '')));
  return result;
}
async function execute(build, order) {
  const requests = [], steps = [], values = {};
  let randomCalls = 0;
  const context = {setTimeout, clearTimeout, console, Math: Object.create(Math)};
  context.self = context;
  context.Math.random = () => ++randomCalls / 10;
  const sandbox = vm.createContext(context);
  const head = {removeChild() {}, appendChild(script) {
    const name = path.basename(script.src);
    requests.push(name);
    script.parentNode = head;
    vm.runInContext(build.files[name], sandbox);
    script.onload?.({type: 'load', target: script});
  }};
  context.document = {head, getElementsByTagName: () => [], createElement: () => ({setAttribute() {}})};
  vm.runInContext(build.files['main.js'], sandbox);
  for (const name of order) {
    values[name] = JSON.parse(JSON.stringify((await context['load' + name]()).value));
    const copies = {};
    for (const file of requests) for (const module of build.modulesByFile[file] || []) copies[module] = (copies[module] || 0) + 1;
    steps.push(copies);
  }
  return {values, randomCalls, steps};
}
export default variants.map(variant => ({
  description: `residual reuse ${variant.label}: copies, names and sequential loads`,
  options(context) {
    return {mode: 'production', context: variant.priority ? path.join(context.getSource(),'priority') : context.getSource(), entry: './index.js', cache: false, incremental: false,
      devtool: false, output: {path: context.getDist(variant.label), publicPath: '/', filename: '[name].js', chunkFilename: '[name].js'},
      optimization: {minimize: false, concatenateModules: false, moduleIds: 'named', chunkIds: 'named',
        splitChunks: {chunks: 'all', dedupDepth: variant.depth, minSize: 50, minSizeReduction: 0,
          cacheGroups: variant.priority ? {defaultVendors: {test: /[/\\](m1|m2|p)\.js$/, minChunks: 2, priority: -10, reuseExistingChunk: true}, default: {minChunks: 2, priority: -20, reuseExistingChunk: true}} : variant.name ? {
            default: false, defaultVendors: false,
            first: {test: /[/\\](m1|p)\.js$/, chunks: /^(C|D|E)$/, minChunks: 2, name: 'S', priority: 0},
            target: {test: /[/\\](m1|m2)\.js$/, chunks: /^(A|B|C)$/, minChunks: 2, name: variant.name, priority: 0},
          } : {defaultVendors: false, default: {minChunks: 2, priority: -20, reuseExistingChunk: true}}
        }
      }
    };
  },
  compiler(_context, compiler) { compiler.outputFileSystem = fs; },
  async build(_context, compiler) {
    const previous = process.env.RSPACK_RESIDUAL_REUSE;
    const run = async enabled => {
      process.env.RSPACK_RESIDUAL_REUSE = enabled ? '1' : '0';
      const stats = await new Promise((resolve,reject) => compiler.run((error,stats) => error ? reject(error) : resolve(stats)));
      assert.equal(stats.hasErrors(),false,stats.toString());
      const compilation = stats.compilation;
      const chunks = [...compilation.chunks].map(chunk => ({name: chunk.name, groups: [...chunk.groupsIterable].map(group=>group.name).sort(), files: [...chunk.files],
        modules: compilation.chunkGraph.getChunkModules(chunk).map(module => path.basename(module.nameForCondition?.() || ''))}));
      const files = Object.fromEntries(Object.keys(compilation.assets).filter(name=>name.endsWith('.js')).map(name=>[name,fs.readFileSync(path.join(compiler.outputPath,name),'utf8')]));
      const modulesByFile = Object.fromEntries(chunks.flatMap(chunk=>chunk.files.map(file=>[file,chunk.modules])));
      return {hash: stats.hash, chunks, files, modulesByFile};
    };
    try {
      const baseline = await run(false), changed = await run(true);
      for (const module of ['m1.js','m2.js']) {
        const expected = variant.priority ? (module==='m1.js' ? 3 : 1) : 3;
        assert.equal(baseline.chunks.filter(chunk=>chunk.modules.includes(module)).length,expected);
        assert.equal(changed.chunks.filter(chunk=>chunk.modules.includes(module)).length,2);
      }
      if (variant.priority) {
        // Recovery belongs to the existing high-priority queue. It must run
        // before the lower-priority group can combine m2 and m3 across ABC.
        const recovered = changed.chunks.find(chunk=>chunk.modules.includes('m1.js')&&chunk.modules.includes('m2.js'));
        assert.deepEqual(recovered.groups,['A','B']);
        assert.equal(baseline.chunks.filter(chunk=>chunk.modules.includes('m3.js')).length,1);
        assert.equal(changed.chunks.filter(chunk=>chunk.modules.includes('m3.js')).length,3);
        // Immediate priority ordering does not guarantee fewer downloads across
        // multiple entries: A then C downloads two copies of m2 instead of one.
        const before = await execute(baseline,'AC'), after = await execute(changed,'AC');
        assert.equal(before.steps[1]['m2.js'],1);
        assert.equal(after.steps[1]['m2.js'],2);
      }
      if (variant.name) {
        const target = changed.chunks.find(chunk=>chunk.name==='T');
        assert.deepEqual(target.groups,['A','B']);
        assert.deepEqual(target.modules.sort(),['m1.js','m2.js']);
      }
      for (const order of orders()) {
        const a = await execute(baseline,order), b = await execute(changed,order);
        assert.deepEqual(b.values,a.values);
        assert.equal(b.randomCalls,a.randomCalls);
        for (let step=0;step<a.steps.length;step++) {
          // All variants preserve the required module set. The priority variant
          // explicitly covers the different partitioning described above.
          assert.deepEqual(Object.keys(b.steps[step]).sort(),Object.keys(a.steps[step]).sort());
          if (!variant.priority) {
            for (const [module,count] of Object.entries(b.steps[step])) assert(count<=(a.steps[step][module]||0),`${variant.label} ${order} ${step} adds ${module}`);
          }
        }
      }
      const repeated = await run(true);
      assert.deepEqual(repeated.files,changed.files);
    } finally {
      if (previous===undefined) delete process.env.RSPACK_RESIDUAL_REUSE;
      else process.env.RSPACK_RESIDUAL_REUSE=previous;
    }
  }
}));
