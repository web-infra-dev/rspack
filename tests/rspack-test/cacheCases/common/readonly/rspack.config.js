let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  context: __dirname,
  cache: {
    type: 'persistent',
  },
  module: {
    rules: [
      {
        test: /readonly\/file\.js/,
        use: {
          loader: './loader.js',
          options: { count: 0 },
        },
      },
    ],
  },
  plugins: [
    {
      apply(compiler) {
        let shouldRebuildFile = true;
        if (index == 0) {
          compiler.options.cache.readonly == false;
          shouldRebuildFile = true;
        } else if (index == 1) {
          compiler.options.cache.readonly == true;
          shouldRebuildFile = true;
        } else if (index == 2) {
          compiler.options.cache.readonly == true;
          shouldRebuildFile = false;
        } else if (index == 3) {
          compiler.options.cache.readonly == false;
          shouldRebuildFile = true;
        } else if (index == 4) {
          compiler.options.cache.readonly == true;
          shouldRebuildFile = false;
        }

        const loaderOptions = compiler.options.module.rules[0].use.options;
        compiler.hooks.done.tap('PLUGIN', function () {
          if (shouldRebuildFile) {
            expect(loaderOptions.count).toBe(1);
          } else {
            expect(loaderOptions.count).toBe(0);
          }
          // reset
          loaderOptions.count = 0;
          index++;
        });
      },
    },
  ],
};
