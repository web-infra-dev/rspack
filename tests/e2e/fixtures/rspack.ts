import path from 'node:path';
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
import { createRequire } from 'node:module';
import { expect } from './base';

const require = createRequire(import.meta.url);

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
  constructor(
    projectDir: string,
    handleRspackConfig: (config: Configuration) => Configuration,
  ) {
    const configPath = path.resolve(projectDir, 'rspack.config.cjs');
    this.config = handleRspackConfig(require(configPath));
    delete require.cache[configPath];
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
    const DevServerConstructor = RspackDevServer;
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
    this.devServer = new DevServerConstructor(
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

export type RspackOptions = {
  rspackConfig: {
    handleConfig(config: Configuration): Configuration;
    basePort: number;
  };
};

export type RspackFixtures = RspackOptions & { rspack: Rspack };

export const rspackFixtures = (): Fixtures<
  RspackFixtures,
  PlaywrightFixture & PathInfoFixtures
> => {
  return {
    rspackConfig: {
      basePort: process.env.RSPACK_E2E_INCREMENTAL === 'true' ? 8200 : 8000,
      handleConfig(config) {
        if (process.env.RSPACK_E2E_INCREMENTAL === 'true') {
          if (config.incremental == undefined) {
            config.incremental = true;
          }
          const cache = config.cache;
          if (typeof cache === 'object' && cache.type === 'persistent') {
            cache.storage = {
              type: 'filesystem',
              ...cache.storage,
              location: 'node_modules/.cache/incremental',
            };
          }
        }
        return config;
      },
    },
    rspack: [
      async ({ page, pathInfo, rspackConfig }, use) => {
        const { tempProjectDir } = pathInfo;
        const port =
          rspackConfig.basePort + Number(process.env.RSTEST_WORKER_ID);
        const rspack = new Rspack(tempProjectDir, (config) => {
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

          return rspackConfig.handleConfig(config);
        });
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
};
