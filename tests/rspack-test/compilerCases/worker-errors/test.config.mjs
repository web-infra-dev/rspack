import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default {
  description: 'resolves async errors after another compiler worker exits',
  async run() {
    await runNodeCase(new URL('./workers.mjs', import.meta.url));
  },
};
