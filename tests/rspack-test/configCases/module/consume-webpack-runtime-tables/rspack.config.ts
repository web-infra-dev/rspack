import { defineConfig } from '@rspack/cli';

const base = defineConfig({
  mode: 'production',
  target: 'node',
  output: { module: true, chunkFormat: 'module' },
});

export default defineConfig([
  {
    ...base,
    name: 'concatenated',
    optimization: { concatenateModules: true, avoidEntryIife: true },
  },
  {
    ...base,
    name: 'not-concatenated',
    optimization: { concatenateModules: false, avoidEntryIife: true },
  },
]);
