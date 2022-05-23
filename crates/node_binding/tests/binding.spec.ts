import assert from 'assert';
import binding from '..';
import path from 'path';
import log from 'why-is-node-running';
import { RawOptions } from '../binding.d';

describe(
  'binding',
  () => {
    it(
      'work',
      () => {
        const options: RawOptions = {
          entries: [path.resolve(__dirname, './index.js')],
          minify: false,
          entryFileNames: path.resolve(__dirname, 'dist/main.js'),
        };
        const instance = binding.newRspack(JSON.stringify(options));
        binding.build(instance);
        // setTimeout(() => {
        //   log();
        // }, 5000);
      },
    );
  },
);
