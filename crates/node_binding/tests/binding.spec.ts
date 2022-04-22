import assert from 'assert';
import binding from '..';
import path from 'path';
import type { BundleOptions } from '..';
describe('binding', () => {
  it('work', () => {
    const options: BundleOptions = {
      entries: [path.resolve(__dirname, './index.js')],
      minify: false,
      entryFileNames: path.resolve(__dirname, 'dist/main.js'),
    };
    const instance = binding.newRspack(JSON.stringify(options));
    binding.build(instance);
  });
});
