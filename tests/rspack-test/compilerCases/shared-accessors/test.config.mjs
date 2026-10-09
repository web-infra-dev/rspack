import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default [
  {
    description: 'keeps shared accessors valid across rebuild, close and GC',
    async run() {
      await runNodeCase(new URL('./lifecycle.mjs', import.meta.url));
    },
  },
  {
    description:
      'isolates shared accessors and releases them when workers exit',
    async run() {
      await runNodeCase(new URL('./workers.mjs', import.meta.url));
    },
  },
];
