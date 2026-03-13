import path from 'node:path';
import { createRequire } from 'node:module';
import {
  rspack,
  type Compiler,
  type Configuration,
  type RspackOptions,
} from '@rspack/core';
import { RspackDevServer } from '@rspack/dev-server';
import { expect, test } from '@/fixtures';

const require = createRequire(import.meta.url);

function waitForBuild(compiler: Compiler) {
  return new Promise<void>((resolve, reject) => {
    let settled = false;
    const complete = (fn: () => void) => {
      if (settled) return;
      settled = true;
      fn();
    };

    compiler.hooks.done.tap('rsc-mf-e2e-wait-build', () => {
      complete(resolve);
    });

    compiler.hooks.failed.tap('rsc-mf-e2e-wait-build', (error) => {
      complete(() => reject(error));
    });
  });
}

async function closeCompiler(compiler: Compiler) {
  await new Promise<void>((resolve, reject) => {
    compiler.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}

async function startRemoteServer(tempProjectDir: string, remotePort: number) {
  const configPath = path.resolve(tempProjectDir, 'remote.rspack.config.js');
  const loadedConfig = require(configPath) as RspackOptions;
  delete require.cache[configPath];

  const config = {
    ...loadedConfig,
    context: tempProjectDir,
    output: {
      ...(loadedConfig.output || {}),
      path: path.resolve(tempProjectDir, 'dist-remote'),
    },
    devServer: {
      ...(loadedConfig.devServer || {}),
      port: remotePort,
    },
  } as Configuration;

  const compiler = rspack(config);
  const ready = waitForBuild(compiler);
  const devServer = new RspackDevServer(
    config.devServer || ({} as any),
    compiler,
  );
  await devServer.start();
  await ready;

  return {
    async stop() {
      await closeCompiler(compiler);
      await devServer.stop();
    },
  };
}

test('should load layered remote modules over http and preserve singleton scopes', async ({
  page,
  pathInfo,
  rspack,
}) => {
  const hostPort = Number(rspack.compiler.options.devServer?.port);
  const remotePort = hostPort + 100;
  const remoteRequests: string[] = [];

  page.on('request', (request) => {
    const url = request.url();
    if (url.includes(`:${remotePort}/`)) {
      remoteRequests.push(url);
    }
  });

  const remote = await startRemoteServer(pathInfo.tempProjectDir, remotePort);
  try {
    await page.reload();

    await page.waitForSelector('[data-testid="remote-widget"]', {
      timeout: 15000,
    });
    await expect(page.locator('[data-testid="title"]')).toHaveText(
      'host-rsc-mf-e2e',
    );
    await expect(page.locator('[data-testid="layer-info"]')).toHaveText(
      'react-server-components-layer',
    );
    await expect(page.locator('[data-testid="remote-widget"]')).toHaveText(
      'remote-widget-from-http',
    );

    await expect
      .poll(() => remoteRequests.some((url) => url.includes('/remoteEntry.js')))
      .toBe(true);

    const scopeMeta = await page.evaluate(() => (window as any).__RSC_MF_E2E__);
    expect(scopeMeta.hostName).toBe('rsc_host_e2e');
    expect(scopeMeta.hasDefaultReact).toBe(true);
    expect(scopeMeta.hasSsrReact).toBe(true);
    expect(scopeMeta.hasRscReact).toBe(true);
    expect(scopeMeta.defaultProbe).toBe('function');
    expect(scopeMeta.ssrProbe).toBe('function');
    expect(scopeMeta.scopes).toEqual(
      expect.arrayContaining(['default', 'ssr', 'rsc']),
    );
  } finally {
    await remote.stop();
  }
});
