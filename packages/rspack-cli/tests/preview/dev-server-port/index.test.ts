import {
  getRandomPort,
  normalizeStdout,
  runWatch,
} from '../../utils/test-utils';

describe('preview command devServer port', () => {
  it('uses devServer.port when --port is not specified', async () => {
    const port = await getRandomPort();
    const { stdout } = await runWatch(
      __dirname,
      ['preview'],
      {
        killString: /localhost/,
      },
      {
        RSPACK_PREVIEW_TEST_PORT: port.toString(),
      },
    );

    expect(normalizeStdout(stdout)).toContain(`http://localhost:${port}/`);
  });

  it('uses --port before devServer.port', async () => {
    const devServerPort = await getRandomPort();
    const cliPort = await getRandomPort();
    const { stdout } = await runWatch(
      __dirname,
      ['preview', '--port', cliPort.toString()],
      {
        killString: /localhost/,
      },
      {
        RSPACK_PREVIEW_TEST_PORT: devServerPort.toString(),
      },
    );

    const output = normalizeStdout(stdout);

    expect(output).toContain(`http://localhost:${cliPort}/`);
    expect(output).not.toContain(`http://localhost:${devServerPort}/`);
  });
});
