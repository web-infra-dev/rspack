import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: {
      type: 'amd',
      name: 'clientContainer',
      amdContainer: "window['clientContainer']",
    },
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [
    new rspack.BannerPlugin({
      raw: true,
      banner:
        "function define(name, deps, fn) { fn(); }\nconst window = {};\nwindow['clientContainer'] = { define };\n",
    }),
  ],
});
