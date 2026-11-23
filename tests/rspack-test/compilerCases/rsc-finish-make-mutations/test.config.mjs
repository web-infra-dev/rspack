import { runNodeCase } from '@rspack/test-tools/helper/node-case';

const build = new URL('./build.mjs', import.meta.url);

export default [
  {
    name: 'late-action',
    description: 'registers actions introduced by the last finishMake tap',
    async run() {
      await runNodeCase(build, ['late-action']);
    },
  },
  {
    name: 'css-order',
    description: 'keeps root CSS ahead of later entry dependencies across RSC rounds',
    async run() {
      await runNodeCase(build, ['css-order']);
    },
  },
];
