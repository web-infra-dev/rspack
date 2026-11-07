import { defineConfig } from '@rspack/cli';

export default defineConfig([
  {
    name: 'avoidEntryIife-false',
    optimization: { avoidEntryIife: false },
  },
  {
    name: 'avoidEntryIife-true',
    optimization: { avoidEntryIife: true },
  },
]);
