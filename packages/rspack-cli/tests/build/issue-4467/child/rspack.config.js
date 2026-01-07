const path = require('path');

let count = 0;
const config = {
  entry: path.resolve(__dirname, './index.js'),
  plugins: [
    {
      apply(compiler) {
        compiler.hooks.afterCompile.tap('child', (base) => {
          if (count > 1) {
            return;
          }
          const child = base.createChildCompiler('child', {}, []);
          child.runAsChild(() => {
            count += 1;
          });
        });
      },
    },
  ],
};

module.exports = config;
