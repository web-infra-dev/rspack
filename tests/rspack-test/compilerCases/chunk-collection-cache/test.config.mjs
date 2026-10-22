import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default {
  description: 'reuses chunk collection arrays across mutations and compilation lifecycle',
  async run() {
    await runNodeCase(new URL('./case.mjs', import.meta.url));
  },
};
