import {
  getRandomPort,
  normalizeStderr,
  normalizeStdout,
  runAndGetProcess,
  run,
} from '../../utils/test-utils';

describe('issue-13271 serve', () => {
  it.concurrent(
    'uses the enabled compiler devServer options when another compiler disables devServer',
    async () => {
      const child = runAndGetProcess(__dirname, [
        'serve',
        '--config',
        './multi.config.js',
      ]);

      let stdout = '';
      let sawWebListening = false;
      let killTimer: NodeJS.Timeout | undefined;

      const hardTimeout = setTimeout(() => {
        child.kill('SIGKILL');
      }, 8000);

      child.stdout?.on('data', (chunk) => {
        stdout += chunk.toString('utf8');

        if (stdout.includes('"source":"disabled"')) {
          child.kill('SIGKILL');
          return;
        }

        if (
          !sawWebListening &&
          stdout.includes('"compiler":"web"') &&
          stdout.includes('"source":"web"') &&
          stdout.includes('"activeCompiler":"web"')
        ) {
          sawWebListening = true;
          killTimer = setTimeout(() => {
            child.kill('SIGKILL');
          }, 1000);
        }
      });

      const result = await child;
      clearTimeout(hardTimeout);
      if (killTimer) {
        clearTimeout(killTimer);
      }

      const output = normalizeStdout(result.stdout ?? stdout);

      expect(output).toContain('"compiler":"web"');
      expect(output).toContain('"source":"web"');
      expect(output).toContain('"activeCompiler":"web"');
      expect(output).not.toContain('"source":"disabled"');
      expect(output).not.toContain('"activeCompiler":"disabled"');
    },
  );

  it.concurrent(
    'falls back to the first compiler when the only compiler disables devServer',
    async () => {
      const port = await getRandomPort();
      const child = runAndGetProcess(__dirname, [
        'serve',
        '--config',
        './single.config.js',
        `--port=${port}`,
      ]);

      let stdout = '';
      const hardTimeout = setTimeout(() => {
        child.kill('SIGKILL');
      }, 8000);

      child.stdout?.on('data', (chunk) => {
        stdout += chunk.toString('utf8');

        if (stdout.includes('Local:')) {
          child.kill('SIGKILL');
        }
      });

      const result = await child;
      clearTimeout(hardTimeout);

      const output = normalizeStdout(result.stdout ?? stdout);

      expect(output).toContain('Local:');
      expect(output).not.toContain(
        "Cannot create property 'hot' on boolean 'false'",
      );
    },
  );
});
