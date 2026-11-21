import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default {
  description: 'reports a watched file removed during the initial directory scan',
  run() {
    return runNodeCase(new URL('./remove-during-scan.mjs', import.meta.url));
  },
};
