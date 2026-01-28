import path from 'node:path';
import type {
  Compiler,
  DevServer,
  MultiCompiler,
  MultiRspackOptions,
  RspackOptions,
} from '@rspack/core';
import { rspack } from '@rspack/core';
// @ts-ignore
import type { RspackDevServer as RspackDevServerType } from '@rspack/dev-server';
import type { RspackCLI } from '../cli';
import type { RspackCommand } from '../types';
import {
  type CommonOptions,
  commonOptions,
  normalizeCommonOptions,
  setDefaultNodeEnv,
} from '../utils/options';

type PreviewOptions = CommonOptions & {
  port?: string;
  host?: string;
  open?: boolean;
  server?: string;
  publicPath?: string;
};

export class PreviewCommand implements RspackCommand {
  async apply(cli: RspackCLI): Promise<void> {
    const command = cli.program
      .command('preview [dir]', 'run the Rspack server for build output')
      .alias('p');

    commonOptions(command)
      .option('--public-path <path>', 'static resource server path')
      .option('--port <port>', 'preview server port')
      .option('--host <host>', 'preview server host')
      .option('--open', 'open browser')
      // same as devServer.server
      .option('--server <config>', 'Configuration items for the server.');

    command.action(async (dir: string | undefined, options: PreviewOptions) => {
      setDefaultNodeEnv(options, 'production');
      normalizeCommonOptions(options, 'preview');

      let RspackDevServer: new (
        options: DevServer,
        compiler: MultiCompiler | Compiler,
      ) => RspackDevServerType;
      try {
        const devServerModule = await import('@rspack/dev-server');
        RspackDevServer = devServerModule.RspackDevServer;
      } catch (error: unknown) {
        const logger = cli.getLogger();
        if (
          (error as Error & { code?: string })?.code === 'MODULE_NOT_FOUND' ||
          (error as Error & { code?: string })?.code === 'ERR_MODULE_NOT_FOUND'
        ) {
          logger.error(
            'The "@rspack/dev-server" package is required to use the preview command.\n' +
              'Please install it by running:\n' +
              '  pnpm add -D @rspack/dev-server\n' +
              '  or\n' +
              '  npm install -D @rspack/dev-server',
          );
        } else {
          logger.error(
            'Failed to load "@rspack/dev-server":\n' +
              ((error as Error)?.message || String(error)),
          );
        }
        process.exit(1);
      }

      let { config } = await cli.loadConfig(options);
      config = await getPreviewConfig(config, options, dir);
      if (!Array.isArray(config)) {
        config = [config as RspackOptions];
      }

      // find the possible devServer config
      const singleConfig = config.find((item) => item.devServer) || config[0];

      const devServerOptions = singleConfig.devServer as DevServer;

      try {
        const compiler = rspack({ entry: {} });
        if (!compiler) return;
        const server = new RspackDevServer(devServerOptions, compiler);

        await server.start();
      } catch (error) {
        const logger = cli.getLogger();
        logger.error(error);

        process.exit(2);
      }
    });
  }
}

// get the devServerOptions from the config
async function getPreviewConfig(
  item: RspackOptions | MultiRspackOptions,
  options: PreviewOptions,
  dir?: string,
): Promise<RspackOptions | MultiRspackOptions> {
  const DEFAULT_ROOT = 'dist';

  const internalPreviewConfig = async (item: RspackOptions) => {
    // all of the options that a preview static server needs(maybe not all)
    item.devServer = {
      static: {
        directory: dir
          ? path.join(item.context ?? process.cwd(), dir)
          : (item.output?.path ??
            path.join(item.context ?? process.cwd(), DEFAULT_ROOT)),
        publicPath: options.publicPath ?? '/',
      },
      port: options.port ?? 8080,
      proxy: item.devServer?.proxy,
      host: options.host ?? item.devServer?.host,
      open: options.open ?? item.devServer?.open,
      server: options.server ?? item.devServer?.server,
      historyApiFallback: item.devServer?.historyApiFallback,
    };
    return item;
  };

  if (Array.isArray(item)) {
    return Promise.all(item.map(internalPreviewConfig));
  }
  return internalPreviewConfig(item as RspackOptions);
}
