const { build } = require('@rspack/core');
const path = require('path');
async function main() {
  const configPath = path.resolve(module.parent.path, 'webpack.config.js');
  const defaultEntry = {
    entry: {
      main: './example.js',
    },
    context: module.parent.path,
  };
  const config = require(configPath);
  // dirty hack to compatible webpack-examples
  let rules = config?.module?.rules;
  if (rules) {
    for (const rule of rules) {
      if (rule.test && rule.test instanceof RegExp) {
        rule.test = rulte.test.toString();
      }
    }
  }
  await build({ ...defaultEntry, ...config });
}
main();
