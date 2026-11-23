import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default {
  description: 'keeps root CSS ahead of later entry dependencies across RSC rounds',
  async run() {
    await runNodeCase(new URL('./build.mjs', import.meta.url));
  },
};
