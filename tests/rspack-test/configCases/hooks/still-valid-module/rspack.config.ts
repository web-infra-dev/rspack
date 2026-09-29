import { defineConfig } from '@rspack/cli';
import MyStillValidModulePlugin from './plugins/MyStillValidModulePlugin.ts';

const config = defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [new MyStillValidModulePlugin()],
});

export default config;
