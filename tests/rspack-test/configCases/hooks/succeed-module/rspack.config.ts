import { defineConfig } from '@rspack/cli';
import MySucceedModulePlugin from './plugins/MySucceedModulePlugin.ts';

const config = defineConfig({
  context: import.meta.dirname,
  mode: 'development',
  entry: {
    main: './src/index.js',
  },
  plugins: [new MySucceedModulePlugin()],
});

export default config;
