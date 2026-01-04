import { runWatch } from '../../utils/test-utils';

describe('issue-11009 serve', () => {
  it.concurrent(
    'should set config path to persistent cache build dependencies',
    async () => {
      if (process.env.WASM) {
        // skip when test wasm
        return;
      }

      const { stdout } = await runWatch(__dirname, ['serve'], {
        killString: /localhost/,
      });

      expect(stdout).toContain('===buildDependencies===');
    },
  );
});
