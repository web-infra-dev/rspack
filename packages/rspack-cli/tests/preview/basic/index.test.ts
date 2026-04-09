import {
  getRandomPort,
  normalizeStderr,
  normalizeStdout,
  runWatch,
} from '../../utils/test-utils';

describe('should run preview command as expected', () => {
  it.concurrent('should work', async () => {
    const port = await getRandomPort();
    const { stdout, stderr } = await runWatch(
      __dirname,
      ['preview', '--port', port.toString()],
      {
        killString: /localhost/,
      },
    );

    expect(normalizeStdout(stdout)).toMatch(
      /Local:\s+http:\/\/localhost:\d+\//,
    );
    expect(normalizeStderr(stderr)).not.toMatch(/<w>|\bwarn(?:ing)?\b/i);
  });
});
