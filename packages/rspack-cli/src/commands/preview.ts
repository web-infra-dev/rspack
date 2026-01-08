import path from 'node:path';
import type {
  DevServer,
  MultiRspackOptions,
  RspackOptions,
} from '@rspack/core';
import type { RspackCLI } from '../cli';
import type { RspackCommand } from '../types';
import {
  type CommonOptions,
  commonOptions,
  normalizeCommonOptions,
  setDefaultNodeEnv,
} from '../utils/options';
import { rspack } from '../utils/rspackCore';

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

      const packageName = '@rspack/dev-server';
      try {
        require.resolve(packageName);
      } catch {
        const logger = cli.getLogger();
        logger.warn(
          `Package "${packageName}" is not installed. Please install it before using "rspack preview" command.`,
        );
        process.exit(2);
      }

      // Lazy import @rspack/dev-server to avoid loading it on build mode
      const { RspackDevServer } = await import('@rspack/dev-server');

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
