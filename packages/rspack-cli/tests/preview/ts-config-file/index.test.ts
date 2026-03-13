import {
  getRandomPort,
  normalizeStdout,
  runWatch,
} from '../../utils/test-utils';

describe('should run preview command with ts config file as expected', () => {
  it.concurrent('should work', async () => {
    const port = await getRandomPort();
    const { stdout } = await runWatch(
      __dirname,
      ['preview', '--port', port.toString()],
      {
        killString: /localhost/,
      },
    );

    expect(normalizeStdout(stdout)).toMatch(
      /Local:\s+http:\/\/localhost:\d+\//,
    );
  });
});
