import { defineConfig, definePlugin } from '@rspack/cli';

const testPlugin = definePlugin(function () {
  let counter = 1;
  this.hooks.compilation.tap('TestPlugin', (compilation) => {
    const nr = counter++;
    compilation.hooks.needAdditionalPass.tap('TestPlugin', function () {
      if (nr < 5) return true;
    });
  });
});

export default defineConfig({
  plugins: [testPlugin],
});
