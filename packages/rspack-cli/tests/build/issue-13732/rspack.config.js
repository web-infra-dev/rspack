const path = require('path');
const PLUGIN_NAME = 'MyPlugin';

class MyPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, () => {
      console.log('MyPlugin: The Rspack build process is starting!');
    });
  }
}

module.exports = {
  mode: 'development',
  entry: './entry.js',
  output: {
    clean: true,
    path: path.resolve(__dirname, 'dist'),
  },
  extends: './base.config.js',
  // Override or add to the base configuration
  output: {
    filename: '[name].bundle.js',
  },
  plugins: [
    // "...", // uncomment this to inherit plugins, but it's also a type error
    new MyPlugin(),
  ],
};
