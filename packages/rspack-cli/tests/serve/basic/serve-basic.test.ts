import { normalizeStderr, runWatch } from '../../utils/test-utils';

describe('basic serve usage', () => {
  it('should work', async () => {
    const { stderr } = await runWatch(__dirname, ['serve'], {
      killString: /localhost/,
    });

    expect(normalizeStderr(stderr)).toContain('Project is running at');
  });
});
