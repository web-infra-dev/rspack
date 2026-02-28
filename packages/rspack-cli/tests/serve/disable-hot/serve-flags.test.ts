import { normalizeStdout, runWatch } from '../../utils/test-utils';

test('should allow to disable HMR via `server.hot`', async () => {
  const { stdout } = await runWatch(__dirname, ['serve'], {
    killString: /localhost/,
  });

  expect(normalizeStdout(stdout)).toContain('{"hot":false}');
});
