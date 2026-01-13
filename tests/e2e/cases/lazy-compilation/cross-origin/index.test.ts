import http from 'node:http';
import path from 'node:path';
import { setTimeout } from 'node:timers/promises';
import { test as base, expect } from '@playwright/test';
import fs from 'fs-extra';
import { type Compiler, type Configuration, rspack } from '@rspack/core';
import { RspackDevServer } from '@rspack/dev-server';

const tempDir = path.resolve(__dirname, '../../temp');

// Create a separate lazy compilation server on a different port (cross-origin)
function createLazyCompilationServer(
  compiler: Compiler,
  port: number,
): Promise<http.Server> {
  return new Promise((resolve, reject) => {
    const middleware = rspack.lazyCompilationMiddleware(compiler);
    const server = http.createServer((req, res) => {
      middleware(req, res, () => {
        res.writeHead(404);
        res.end('Not Found');
      });
    });

    server.listen(port, () => {
      resolve(server);
    });

    server.on('error', reject);
  });
}

const test = base.extend<{
  crossOriginSetup: {
    frontendPort: number;
    lazyCompilationPort: number;
  };
}>({
  crossOriginSetup: [
    async ({ page }, use, testInfo) => {
      const workerId = String(testInfo.workerIndex);
      const testProjectDir = path.dirname(testInfo.file);
      const tempProjectDir = path.join(tempDir, `cross-origin-${workerId}`);

      // Copy test project to temp directory
      if (await fs.exists(tempProjectDir)) {
        await fs.remove(tempProjectDir);
      }
      await fs.copy(testProjectDir, tempProjectDir);

      // Clear require cache for temp directory
      for (const modulePath of Object.keys(require.cache)) {
        if (modulePath.startsWith(tempProjectDir)) {
          delete require.cache[modulePath];
        }
      }

      // Use different ports for frontend and lazy compilation server
      const basePort = 8500;
      const frontendPort = basePort + testInfo.workerIndex;
      const lazyCompilationPort = frontendPort + 100;

      // Load and modify config
      const configPath = path.resolve(tempProjectDir, 'rspack.config.js');
      const config: Configuration = require(configPath);
      delete require.cache[configPath];

      config.context = tempProjectDir;
      config.output = {
        ...config.output,
        path: path.resolve(tempProjectDir, 'dist'),
      };
      config.devServer = {
        ...config.devServer,
        port: frontendPort,
      };
      // Set cross-origin serverUrl
      config.lazyCompilation = {
        ...(typeof config.lazyCompilation === 'object'
          ? config.lazyCompilation
          : {}),
        entries: false,
        imports: true,
        serverUrl: `http://127.0.0.1:${lazyCompilationPort}`,
      };

      // Create compiler
      const compiler = rspack(config);

      // Start lazy compilation server on a different port (cross-origin)
      const lazyServer = await createLazyCompilationServer(
        compiler,
        lazyCompilationPort,
      );

      // Start dev server (frontend) - without lazy compilation middleware
      // since we're running it on a separate server
      const devServer = new RspackDevServer(
        {
          ...config.devServer,
          port: frontendPort,
        },
        compiler,
      );
      await devServer.start();

      // Wait for initial build
      await new Promise<void>((resolve) => {
        compiler.hooks.done.tap('test', () => resolve());
      });

      // Navigate to frontend
      await page.goto(`http://localhost:${frontendPort}`);

      await use({ frontendPort, lazyCompilationPort });

      // Cleanup
      await new Promise<void>((res, rej) => {
        compiler.close((err) => (err ? rej(err) : res()));
      });
      await devServer.stop();
      await new Promise<void>((resolve) => {
        lazyServer.close(() => resolve());
      });
      await fs.remove(tempProjectDir);
    },
    { auto: true },
  ],
});

test('should work with cross-origin lazy compilation using simple POST request', async ({
  page,
  crossOriginSetup,
}) => {
  const { lazyCompilationPort } = crossOriginSetup;

  // Set up request interception to verify the request format
  const requests: { method: string; contentType: string; body: string }[] = [];

  page.on('request', (request) => {
    const url = request.url();
    if (url.includes(`${lazyCompilationPort}`)) {
      requests.push({
        method: request.method(),
        contentType: request.headers()['content-type'] || '',
        body: request.postData() || '',
      });
    }
  });

  await page.waitForSelector('button:has-text("Click me")');

  // Click the button to trigger dynamic import (cross-origin lazy compilation request)
  await page.getByText('Click me').click();

  // Wait for the component to appear - this confirms the cross-origin request worked
  await page.waitForSelector('div:has-text("CrossOriginComponent")', {
    timeout: 10000,
  });

  const componentCount = await page.getByText('CrossOriginComponent').count();
  expect(componentCount).toBe(1);

  // Verify the request was a simple POST request with text/plain content type
  const lazyRequest = requests.find((r) => r.method === 'POST');
  expect(lazyRequest).toBeDefined();
  expect(lazyRequest!.contentType).toBe('text/plain');
  // The body should be newline-separated module IDs, not JSON
  expect(lazyRequest!.body).not.toMatch(/^\[/); // Not a JSON array
});
