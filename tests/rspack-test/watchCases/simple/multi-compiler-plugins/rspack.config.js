const events = [];
const state = { applied: 0, thisIsCompiler: false, optionsPlugins: false };

class MultiCompilerRecorderPlugin {
  apply(compiler) {
    state.applied++;
    compiler.hooks.watchRun.tap('MultiCompilerRecorderPlugin', (child) => {
      events.push(`watchRun ${child.name}`);
    });
    compiler.hooks.done.tap('MultiCompilerRecorderPlugin', (stats) => {
      events.push(`done ${stats.stats.length}`);
    });
  }
}

function recordThis(compiler) {
  state.thisIsCompiler = this === compiler && compiler.compilers.length === 2;
  state.optionsPlugins = compiler.options.plugins === plugins;
}

const plugins = [new MultiCompilerRecorderPlugin(), recordThis, false, null];

module.exports = Object.assign(
  [
    {
      name: 'a',
      entry: './index.js',
      output: { clean: false, filename: 'bundle.js' },
    },
    {
      name: 'b',
      entry: './b.js',
      output: { clean: false, filename: 'b-bundle.js' },
    },
  ],
  { plugins, events, state },
);
