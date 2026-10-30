import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default ['close', 'terminate'].map((mode) => ({
  name: mode,
  description: `deletes references in their owning compiler worker after ${mode}`,
  async run() {
    await runNodeCase(new URL('./workers.mjs', import.meta.url), [mode]);
  },
}));
