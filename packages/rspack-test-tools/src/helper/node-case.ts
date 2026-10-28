import { spawn } from 'node:child_process';

const resultPrefix = '__RSPACK_NODE_CASE_RESULT__:';

// Keep the bootstrap independent of the test runner and its native handles.
const bootstrap = `
const prefix = ${JSON.stringify(resultPrefix)};
try {
  const { default: run } = await import(process.argv[1]);
  const result = await run(...JSON.parse(process.argv[2]));
  const encoded = JSON.stringify(result === undefined ? null : result, (_key, value) => {
    if (['undefined', 'function', 'symbol', 'bigint'].includes(typeof value) ||
        (typeof value === 'number' && !Number.isFinite(value))) {
      throw new Error('Node case result must be JSON serializable');
    }
    return value;
  });
  process.stdout.write('\\n' + prefix + encoded + '\\n');
} catch (error) {
  console.error(error);
  process.exitCode = 1;
}
`;

/** Run a standalone default-exported scenario and wait for natural process exit. */
export function runNodeCase<T = unknown>(
  moduleURL: URL,
  args: unknown[] = [],
): Promise<T> {
  if (moduleURL.protocol !== 'file:') {
    throw new Error('Node case must use an absolute file URL');
  }
  const encodedArgs = JSON.stringify(args, (_key, value) => {
    if (
      ['undefined', 'function', 'symbol', 'bigint'].includes(typeof value) ||
      (typeof value === 'number' && !Number.isFinite(value))
    ) {
      throw new Error('Node case arguments must be JSON serializable');
    }
    return value;
  });
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      [
        '--expose-gc',
        '--input-type=module',
        '--eval',
        bootstrap,
        moduleURL.href,
        encodedArgs,
      ],
      { stdio: ['ignore', 'pipe', 'pipe'] },
    );
    let stdout = '';
    let stderr = '';
    let timedOut = false;
    let spawnError: Error | undefined;
    const timeout = setTimeout(() => {
      timedOut = true;
      child.kill('SIGKILL');
    }, 20_000);
    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');
    const onStdout = (chunk: string) => {
      stdout += chunk;
    };
    const onStderr = (chunk: string) => {
      stderr += chunk;
    };
    const onError = (error: Error) => {
      spawnError = error;
    };
    child.stdout.on('data', onStdout);
    child.stderr.on('data', onStderr);
    child.on('error', onError);
    child.once('close', (code, signal) => {
      clearTimeout(timeout);
      child.stdout.off('data', onStdout);
      child.stderr.off('data', onStderr);
      child.off('error', onError);
      try {
        if (timedOut) throw new Error('timed out after 20s');
        if (spawnError) throw spawnError;
        if (code !== 0)
          throw new Error(`exited with code ${code}, signal ${signal}`);
        const results = stdout
          .split(/\r?\n/)
          .filter((line) => line.startsWith(resultPrefix));
        if (results.length !== 1)
          throw new Error(`expected one result, received ${results.length}`);
        resolve(JSON.parse(results[0].slice(resultPrefix.length)) as T);
      } catch (error) {
        reject(
          new Error(
            `${moduleURL.href} ${encodedArgs}: ${String(error)}\nstdout:\n${stdout}\nstderr:\n${stderr}`,
          ),
        );
      }
    });
  });
}
