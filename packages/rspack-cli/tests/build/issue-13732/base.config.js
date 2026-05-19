const PLUGIN_NAME = 'MyBasePlugin';

export class MyBasePlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, () => {
      console.log('MyBasePlugin: The Rspack build process is starting!');
    });
  }
}

module.exports = {
  plugins: [new MyBasePlugin()],
};
