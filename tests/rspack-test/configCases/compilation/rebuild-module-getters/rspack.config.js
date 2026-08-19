const pluginName = 'plugin';

class Plugin {
  apply(compiler) {
    let initial = true;
    compiler.hooks.compilation.tap(pluginName, (compilation) => {
      compilation.hooks.finishModules.tapPromise(
        pluginName,
        async (modules) => {
          if (!initial) {
            return;
          }
          initial = false;
          const oldModule = [...modules].find((item) =>
            item.resource.endsWith('a.js'),
          );
          if (!oldModule) {
            throw new Error('module not found');
          }
          await new Promise((res, rej) => {
            compilation.rebuildModule(oldModule, function (err, m) {
              if (err) {
                rej(err);
              } else {
                res(m);
              }
            });
          });
        },
      );
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  module: {
    rules: [
      {
        test: /a\.js$/,
        use: [
          {
            loader: './loader',
          },
        ],
      },
    ],
  },
  plugins: [new Plugin()],
};
