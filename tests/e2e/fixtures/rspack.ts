import path from 'node:path';
import { pathToFileURL } from 'node:url';
import type { Fixtures } from 'rstack/test';
import type { PlaywrightFixture } from '@rstest/playwright';
import {
  type Compiler,
  type Configuration,
  rspack,
  type RspackOptions as RspackConfig,
} from '@rspack/core';
import { RspackDevServer } from '@rspack/dev-server';
import type { PathInfoFixtures } from './pathInfo';
import { expect } from './base';

// Test plugins expose these fields, and dev middleware supplies a synchronous filesystem.
type FixtureCompiler = Compiler & {
  __modules: string[];
  __sharedObj: { time: number; useFullEntry: boolean };
  outputFileSystem: NonNullable<Compiler['outputFileSystem']> &
    Pick<typeof import('node:fs'), 'readFileSync' | 'readdirSync'>;
};

class Rspack {
  private config: RspackConfig;
  projectDir: string;
  outDir: string;
  compiler!: FixtureCompiler;
  devServer!: RspackDevServer;
  private onDone: Array<() => void> = [];
  constructor(projectDir: string, config: Configuration) {
    this.config = config;
    this.projectDir = projectDir;
    this.outDir = this.config.output!.path!;
  }

  // waiting for build done, not hmr done
  async waitingForBuild() {
    if (!this.compiler.watching?.running) {
      return;
    }

    return new Promise<void>((resolve) => {
      this.onDone.push(resolve);
    });
  }

  async stop() {
    await new Promise<void>((res, rej) => {
      this.compiler.close(function (err) {
        if (err) {
          rej(err);
        } else {
          res();
        }
      });
    });
    await this.devServer.stop();
  }

  async start() {
    const compiler = rspack(this.config);
    this.compiler = compiler as FixtureCompiler;
    this.compiler.hooks.done.tap('rspack_fixture', () => {
      const onDone = this.onDone;
      this.onDone = [];
      for (const item of onDone) {
        item();
      }
    });
    if (compiler.options.lazyCompilation) {
      const middleware = rspack.lazyCompilationMiddleware(compiler);
      const devServerOptions = compiler.options.devServer || {};
      compiler.options.devServer = devServerOptions;
      const setupMiddlewares = devServerOptions.setupMiddlewares;
      devServerOptions.setupMiddlewares = (middlewares, server) => {
        const old = setupMiddlewares
          ? setupMiddlewares(middlewares, server)
          : middlewares;
        return [middleware, ...old];
      };
    }
    this.devServer = new RspackDevServer(
      compiler.options.devServer ?? ({} as any),
      compiler,
    );
    await this.devServer.start();
    await this.waitingForBuild();
  }

  async reboot() {
    await this.stop();
    await this.start();
  }
}

export type RspackFixtures = { rspack: Rspack };

export const rspackFixtures: Fixtures<
  RspackFixtures,
  PlaywrightFixture & PathInfoFixtures
> = {
  rspack: [
    async ({ page, pathInfo }, use) => {
      const { tempProjectDir } = pathInfo;
      const port = 8000 + Number(process.env.RSTEST_WORKER_ID);
      const configPath = path.join(tempProjectDir, 'rspack.config.js');
      const { default: config }: { default: Configuration } = await import(
        /* webpackIgnore: true */ pathToFileURL(configPath).href
      );
      // rewrite port
      if (!config.devServer) {
        config.devServer = {};
      }
      config.devServer.port = port;

      // set default context
      if (!config.context) {
        config.context = tempProjectDir;
      }

      // set default output path
      if (!config.output) {
        config.output = {};
      }
      config.output.path = path.resolve(tempProjectDir, 'dist');

      const rspack = new Rspack(tempProjectDir, config);
      await rspack.start();

      await page.goto(`http://localhost:${port}`);
      // Initial HTML can render before the client connects. Wait before tests
      // edit files, otherwise the browser can miss the first HMR notification.
      await expect
        .poll(() => rspack.devServer.webSocketServer?.clients.length ?? 0)
        .toBeGreaterThan(0);

      await use(rspack);

      await rspack.stop();
    },
    { auto: true },
  ],
};
