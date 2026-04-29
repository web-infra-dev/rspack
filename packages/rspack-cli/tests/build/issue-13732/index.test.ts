import { resolve } from 'path';
import { readFile, run } from '../../utils/test-utils';

describe('issue-11009 build', () => {
  it.concurrent(
    'should set config path to persistent cache build dependencies',
    async () => {
      if (process.env.WASM) {
        // skip when test wasm
        return;
      }
      const { stdout, stderr } = await run(__dirname, []);
      expect(stdout).toContain(
        'MyBasePlugin: The Rspack build process is starting!',
      );
      expect(stdout).toContain(
        'MyPlugin: The Rspack build process is starting!',
      );
    },
  );
});
