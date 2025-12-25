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
      const { stdout } = await run(__dirname, []);
      const mainJs = await readFile(
        resolve(__dirname, 'dist/main.js'),
        'utf-8',
      );
      expect(mainJs).toContain('entry-issue-11009');
      expect(stdout).toContain('===buildDependencies===');
    },
  );
});
