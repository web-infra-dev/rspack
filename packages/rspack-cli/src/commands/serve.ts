import type { Compiler, MultiRspackOptions } from '@rspack/core';
import type { RspackCLI } from '../cli';
import { DEFAULT_SERVER_HOT } from '../constants';
import type { RspackCommand } from '../types';
import {
  type CommonOptionsForBuildAndServe,
  commonOptions,
  commonOptionsForBuildAndServe,
  normalizeCommonOptions,
  setDefaultNodeEnv,
} from '../utils/options';

type ServerOptions = CommonOptionsForBuildAndServe & {
  hot?: boolean | 'only';
  port?: number | string;
  host?: string;
};

function normalizeHotOption(
  value: boolean | 'true' | 'false' | 'only' | undefined,
): ServerOptions['hot'] {
  if (value === 'false') {
    return false;
  }
  if (value === 'true') {
    return true;
  }
  return value;
}

export class ServeCommand implements RspackCommand {
  async apply(cli: RspackCLI): Promise<void> {
    const command = cli.program
      .command('serve', 'run the rspack dev server.')
      .alias('server')
      .alias('s')
      .alias('dev');

    commonOptionsForBuildAndServe(commonOptions(command))
      .option('--hot [mode]', 'enables hot module replacement')
      .option('--port <port>', 'allows to specify a port to use')
      .option('--host <host>', 'allows to specify a hostname to use');

    command.action(async (cliOptions: ServerOptions) => {
      setDefaultNodeEnv(cliOptions, 'development');
      normalizeCommonOptions(cliOptions, 'serve');
      cliOptions.hot = normalizeHotOption(cliOptions.hot);

      // Lazy import @rspack/dev-server to avoid loading it on build mode
      let RspackDevServer: any;
      try {
        const devServerModule = await import('@rspack/dev-server');
        RspackDevServer = devServerModule.RspackDevServer;
      } catch (error: any) {
        const logger = cli.getLogger();
        if (
          error?.code === 'MODULE_NOT_FOUND' ||
          error?.code === 'ERR_MODULE_NOT_FOUND'
        ) {
          logger.error(
            'The "@rspack/dev-server" package is required to use the serve command.\n' +
            'Please install it by running:\n' +
            '  pnpm add -D @rspack/dev-server\n' +
            '  or\n' +
            '  npm install -D @rspack/dev-server',
          );
        } else {
          logger.error(
            'Failed to load "@rspack/dev-server":\n' +
            (error?.message || String(error)),
          );
        }
        process.exit(1);
      }

      const userConfig = await cli.buildCompilerConfig(cliOptions, 'serve');
      const compiler = await cli.createCompiler(userConfig);
      if (!compiler) {
        return;
      }
      const isMultiCompiler = cli.isMultipleCompiler(compiler);

      const compilers = isMultiCompiler ? compiler.compilers : [compiler];
      const userConfigs = isMultiCompiler
        ? (userConfig as MultiRspackOptions)
        : ([userConfig] as MultiRspackOptions);

      const possibleCompilers = compilers.filter(
        (compiler: Compiler) => compiler.options.devServer,
      );

      const usedPorts: number[] = [];
      const servers: any[] = [];

      /**
       * webpack uses an Array of compilerForDevServer,
       * however according to it's doc https://webpack.js.org/configuration/dev-server/#devserverhot
       * It should use only the first one
       *
       * Choose the one for configure devServer
       */
      const compilerForDevServer =
        possibleCompilers.length > 0 ? possibleCompilers[0] : compilers[0];

      /**
       * Rspack relies on devServer.hot to enable HMR
       */
      for (const [index, compiler] of compilers.entries()) {
        const userConfig = userConfigs[index];
        const devServer = (compiler.options.devServer ??= {});
        const isWebAppOnly =
          compiler.platform.web &&
          !compiler.platform.node &&
          !compiler.platform.nwjs &&
          !compiler.platform.electron &&
          !compiler.platform.webworker;

        if (isWebAppOnly && userConfig.lazyCompilation === undefined) {
          compiler.options.lazyCompilation = {
            imports: true,
            entries: false,
          };
        }

        devServer.hot = cliOptions.hot ?? devServer.hot ?? DEFAULT_SERVER_HOT;

        if (devServer.client !== false) {
          if (devServer.client === true || devServer.client == null) {
            devServer.client = {};
          }
          devServer.client = {
            overlay: {
              errors: true,
              warnings: false,
            },
            ...devServer.client,
          };
        }
      }

      const devServerOptions = (compilerForDevServer.options.devServer ??= {});
      const { setupMiddlewares } = devServerOptions;

      const lazyCompileMiddleware = rspack.lazyCompilationMiddleware(compiler);

      devServerOptions.setupMiddlewares = (middlewares, server) => {
        let finalMiddlewares = middlewares;
        if (setupMiddlewares) {
          finalMiddlewares = setupMiddlewares(finalMiddlewares, server);
        }
        return [...finalMiddlewares, lazyCompileMiddleware];
      };

      /**
       * Enable this to tell Rspack that we need to enable React Refresh by default
       */
      devServerOptions.hot =
        cliOptions.hot ?? devServerOptions.hot ?? DEFAULT_SERVER_HOT;
      devServerOptions.host = cliOptions.host || devServerOptions.host;
      devServerOptions.port = cliOptions.port ?? devServerOptions.port;
      if (devServerOptions.client !== false) {
        if (
          devServerOptions.client === true ||
          devServerOptions.client == null
        ) {
          devServerOptions.client = {};
        }
        devServerOptions.client = {
          overlay: {
            errors: true,
            warnings: false,
          },
          ...devServerOptions.client,
        };
      }

      if (devServerOptions.port) {
        const portNumber = Number(devServerOptions.port);

        if (!Number.isNaN(portNumber)) {
          if (usedPorts.find((port) => portNumber === port)) {
            throw new Error(
              'Unique ports must be specified for each devServer option in your rspack configuration. Alternatively, run only 1 devServer config using the --config-name flag to specify your desired config.',
            );
          }

          usedPorts.push(portNumber);
        }
      }

      try {
        const server = new RspackDevServer(devServerOptions, compiler);
        await server.start();
        servers.push(server);
      } catch (error) {
        const logger = cli.getLogger();
        logger.error(error);

        process.exit(2);
      }
    });
  }
}
