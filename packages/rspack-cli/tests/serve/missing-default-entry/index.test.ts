import {
  normalizeStderr,
  runWatch,
  uniqueDirectoryForTest,
} from '../../utils/test-utils';

describe('serve without a config or default entry', () => {
  it('suggests preview when serving build output may be intended', async () => {
    const cwd = await uniqueDirectoryForTest();
    const { stdout, stderr } = await runWatch(cwd, ['serve'], {
      killString: /Module not found|rspack preview \[dir\]/,
    });

    const output = normalizeStderr(`${stdout}\n${stderr}`);
    expect(output).toContain(
      "No rspack config found and default entry './src' does not exist.",
    );
    expect(output).toContain('use `rspack preview [dir]`');
  });
});
