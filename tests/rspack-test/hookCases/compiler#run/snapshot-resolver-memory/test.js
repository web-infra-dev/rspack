const fs = require('node:fs');
const path = require('node:path');
const { promisify } = require('node:util');
const vm = require('node:vm');
const evaluate = stats => {
  const module = { exports: {} };
  vm.runInNewContext(stats.compilation.assets['main.js'].source(), { module });
  return module.exports;
};
let input;
const metadata = main => {
  const file = path.join(input, 'package.json');
  fs.writeFileSync(file, JSON.stringify({ main }));
  fs.utimesSync(file, 1000000000, 1000000000);
};
module.exports = {
  description: 'should validate resolver snapshots on consecutive runs of one compiler',
  snapshotFileFilter: () => false,
  options(context) {
    input = path.join(context.getDist(), 'input');
    fs.mkdirSync(input, { recursive: true });
    fs.writeFileSync(path.join(input, 'a.js'), 'module.exports = 1;');
    fs.writeFileSync(path.join(input, 'b.js'), 'module.exports = 2;');
    metadata('a.js');
    return {
      context: context.getDist(),
      entry: './input',
      output: { library: { type: 'commonjs2' } },
      incremental: false,
      cache: true,
      experiments: { newCache: { module: false, loader: false } },
      snapshot: { resolve: { hash: true } },
    };
  },
  async compiler(context, compiler) {
    const run = promisify(compiler.run.bind(compiler));
    const first = await run();
    expect(first.hasErrors()).toBe(false);
    expect(evaluate(first)).toBe(1);
    metadata('b.js');
    const second = await run();
    expect(second.hasErrors()).toBe(false);
    expect(evaluate(second)).toBe(2);
  },
};
