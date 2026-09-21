import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

export default defineConfig({
  output: {
    library: {
      type: 'amd-require',
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
        "var nodeRequire = require;\nvar require = function(deps, fn) { fn(); }\nconst window = {};\nwindow['clientContainer'] = { require };\n",
    }),
  ],
});
