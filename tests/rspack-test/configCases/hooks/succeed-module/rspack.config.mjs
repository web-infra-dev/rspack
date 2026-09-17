import MySucceedModulePlugin from './plugins/MySucceedModulePlugin.mjs';

/** @type {import("@rspack/core").Configuration} */
const config = {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [new MySucceedModulePlugin()],
};

export default config;
