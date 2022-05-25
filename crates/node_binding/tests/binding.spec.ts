import assert from 'assert';
import binding from '..';
import path from 'path';
import log from 'why-is-node-running';
import { RawOptions } from '../binding.d';

describe('binding', () => {
  it('work', async () => {
    const options: RawOptions = {
      entries: { main: path.resolve(__dirname, './index.js') },
      // entryFilename: path.resolve(__dirname, 'dist/main.js'),
    };
    const instance = binding.newRspack(JSON.stringify(options));
    await binding.build(instance);
    // setTimeout(() => {
    //   log();
    // }, 5000);
  });
});
