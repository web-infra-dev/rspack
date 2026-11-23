import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default {
  description: 'closes both RSC compilers after an early server finishMake error',
  async run() {
    await runNodeCase(new URL('./build.mjs', import.meta.url));
  },
};
