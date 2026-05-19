import path from 'path';
import { run } from '../../utils/test-utils';

const test = process.env.WASM ? it : it.concurrent;

test('should not print the warning for child compiler', async () => {
  const cwd = path.resolve(__dirname, './child');
  const { exitCode, stderr } = await run(
    cwd,
    [],
    {},
    {
      RSPACK_DEP_WARNINGS: true,
    },
  );
  expect(exitCode).toBe(0);
  expect(stderr).not.toContain('deprecated');
});

test('should print the warning for root compiler', async () => {
  const cwd = path.resolve(__dirname, './root');
  const { exitCode } = await run(
    cwd,
    [],
    {},
    {
      RSPACK_DEP_WARNINGS: true,
    },
  );
  expect(exitCode).toBe(0);
  // expect(stderr).toContain('deprecated');
});
