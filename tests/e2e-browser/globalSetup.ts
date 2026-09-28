import net from 'node:net';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import path from 'node:path';
import { createRequire } from 'node:module';
import { createRsbuild } from 'rstack/app';
import { appConfig } from './app.config.ts';

const require = createRequire(import.meta.url);

function isServerRunning(): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = net.connect(8900, 'localhost');
    const finish = (running: boolean) => {
      socket.destroy();
      resolve(running);
    };
    socket.once('connect', () => finish(true));
    socket.once('error', () => finish(false));
    socket.setTimeout(1000, () => finish(false));
  });
}

export default async function setup() {
  if (await isServerRunning()) {
    if (process.env.CI) {
      throw new Error('Port 8900 is already in use in CI.');
    }
    return;
  }

  // Keep the original production build command and its 60-second startup budget.
  const rs = path.join(
    path.dirname(require.resolve('rstack/package.json')),
    'bin/rs.js',
  );
  await promisify(execFile)(process.execPath, [rs, 'build'], {
    cwd: import.meta.dirname,
    timeout: 60_000,
    env: { ...process.env, NODE_ENV: 'production' },
  });
  const rsbuild = await createRsbuild({
    cwd: import.meta.dirname,
    rsbuildConfig: {
      ...appConfig,
      server: { ...appConfig.server, strictPort: true },
    },
  });
  const { server } = await rsbuild.preview();
  return () => server.close();
}
