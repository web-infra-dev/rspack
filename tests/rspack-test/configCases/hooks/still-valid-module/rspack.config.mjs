import MyStillValidModulePlugin from './plugins/MyStillValidModulePlugin.js';

/** @type {import("@rspack/core").Configuration} */
const config = {
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [new MyStillValidModulePlugin()],
};
export default config;
