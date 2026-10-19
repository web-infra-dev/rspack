import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';

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
export default [false,true].map(named => ({
  description: `incremental candidate updates across two committed deletions: named=${named}`,
  options(context) {
    return {mode:'production',context:context.getSource(),entry:'./index.js',cache:false,incremental:false,devtool:false,
      output:{path:context.getDist(String(named)),publicPath:'/',filename:'[name].js',chunkFilename:'[name].js'},
      optimization:{minimize:false,concatenateModules:false,moduleIds:'named',chunkIds:'named',splitChunks:{chunks:'all',dedupDepth:1,minSize:60,minSizeReduction:0,
        cacheGroups:{default:false,defaultVendors:false,
          first:{test:/[/\\](m1|p)\.js$/,chunks:/^[CEFI]$/,name:'S1',minChunks:2,priority:0},
          second:{test:/[/\\](m2|q)\.js$/,chunks:/^[DGHJ]$/,name:'S2',minChunks:2,priority:0},
          target:{test:/[/\\](m1|m2)\.js$/,chunks:/^[ABCD]$/,name:named?'T':false,minChunks:2,priority:0,reuseExistingChunk:true}
        }}}};
  },
  compiler(_context,compiler) {compiler.outputFileSystem=fs;},
  async build(_context,compiler) {
    const previous=process.env.RSPACK_RESIDUAL_REUSE;
    const run=async enabled=>{
      process.env.RSPACK_RESIDUAL_REUSE=enabled?'1':'0';
      const stats=await new Promise((resolve,reject)=>compiler.run((error,stats)=>error?reject(error):resolve(stats)));
      assert.equal(stats.hasErrors(),false,stats.toString());
      const compilation=stats.compilation;
      const chunks=[...compilation.chunks].map(chunk=>({groups:[...chunk.groupsIterable].map(group=>group.name).sort(),files:[...chunk.files],modules:compilation.chunkGraph.getChunkModules(chunk).map(module=>path.basename(module.nameForCondition?.()||''))}));
      const files=Object.fromEntries(Object.keys(compilation.assets).filter(name=>name.endsWith('.js')).map(name=>[name,fs.readFileSync(path.join(compiler.outputPath,name),'utf8')]));
      return {chunks,files,modulesByFile:Object.fromEntries(chunks.flatMap(chunk=>chunk.files.map(file=>[file,chunk.modules])))};
    };
    try {
      const baseline=await run(false),changed=await run(true);
      assert.equal(baseline.chunks.filter(chunk=>chunk.modules.includes('m1.js')).length,4);
      assert.equal(baseline.chunks.filter(chunk=>chunk.modules.includes('m2.js')).length,4);
      assert.equal(changed.chunks.filter(chunk=>chunk.modules.includes('m1.js')).length,3);
      assert.equal(changed.chunks.filter(chunk=>chunk.modules.includes('m2.js')).length,3);
      const shared=changed.chunks.find(chunk=>chunk.modules.includes('m1.js')&&chunk.modules.includes('m2.js'));
      assert.deepEqual(shared.groups,['A','B']);
      const letters='ABCDEFGHIJ';
      for (const order of [...letters,letters,[...letters].reverse().join(''),'ABGH','GHAB','ACB','BCA']) {
        const a=await execute(baseline,order),b=await execute(changed,order);
        assert.deepEqual(b.values,a.values);
        assert.equal(b.randomCalls,a.randomCalls);
        for (let step=0;step<a.steps.length;step++) for (const [module,count] of Object.entries(b.steps[step])) assert(count<=(a.steps[step][module]||0),`${named} ${order} ${step} adds ${module}`);
      }
      const repeat=await run(true);assert.deepEqual(repeat.files,changed.files);
    } finally {
      if (previous===undefined) delete process.env.RSPACK_RESIDUAL_REUSE;else process.env.RSPACK_RESIDUAL_REUSE=previous;
    }
  }
}));
