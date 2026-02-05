const ServerPlugin = require("./server");

const serverPlugin = new ServerPlugin(9991);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'web',
  entry: './index.js',
  optimization: {
    moduleIds: 'named'
  },
  plugins: [
    serverPlugin,
    {
      apply(compiler){
        const startTime = Date.now();

        compiler.hooks.beforeRun.tap('debug-lifecycle', () => {
          console.log(`[LIFECYCLE] beforeRun at ${Date.now() - startTime}ms`);
        });

        compiler.hooks.run.tap('debug-lifecycle', () => {
          console.log(`[LIFECYCLE] run at ${Date.now() - startTime}ms`);
        });

        compiler.hooks.make.tapAsync('debug-lifecycle', (compilation, callback) => {
          console.log(`[LIFECYCLE] make started at ${Date.now() - startTime}ms`);
          callback();
        });

        compiler.hooks.afterCompile.tapAsync('debug-lifecycle', (compilation, callback) => {
          console.log(`[LIFECYCLE] afterCompile at ${Date.now() - startTime}ms`);
          console.log(`[LIFECYCLE] modules count: ${compilation.modules.size}`);
          console.log(`[LIFECYCLE] errors count: ${compilation.errors.length}`);
          callback();
        });

        compiler.hooks.done.tap('test', (s)=>{
          console.log(`[LIFECYCLE] done at ${Date.now() - startTime}ms`);
          console.log(s.toJson())

        })
      }
    }
  ],
  experiments: {
    buildHttp: {
      frozen: false,
      allowedUris: [
        "http://localhost:9991/"
      ],
    }
  }
};
