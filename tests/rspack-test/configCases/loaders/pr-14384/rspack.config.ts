import { defineConfig } from '@rspack/cli';
import PluginWithLoader from './PluginWithLoader.mjs';

export default defineConfig({
  plugins: [new PluginWithLoader()],
});
