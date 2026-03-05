import { normalizeStdout, runWatch } from '../../utils/test-utils';

describe('basic serve usage', () => {
  it('should work', async () => {
    const { stdout } = await runWatch(__dirname, ['serve'], {
      killString: /localhost/,
    });

    expect(normalizeStdout(stdout)).toMatch(
      /Local:\s+http:\/\/localhost:\d+\//,
    );
  });
});
