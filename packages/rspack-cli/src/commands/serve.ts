import type { Compiler } from "@rspack/core";
import type { RspackDevServer as RspackDevServerType } from "@rspack/dev-server";
import type { RspackCLI } from "../cli";
import { DEFAULT_SERVER_HOT } from "../constants";
import type { RspackCommand } from "../types";
import {
	type CommonOptionsForBuildAndServe,
	commonOptions,
	commonOptionsForBuildAndServe,
	normalizeCommonOptions,
	setDefaultNodeEnv
} from "../utils/options";
import { rspack } from "../utils/rspackCore";

type ServerOptions = CommonOptionsForBuildAndServe & {
	hot?: boolean | "only";
	port?: number | string;
	host?: string;
};

function normalizeHotOption(
	value: boolean | "true" | "false" | "only" | undefined
): ServerOptions["hot"] {
	if (value === "false") {
		return false;
	}
	if (value === "true") {
		return true;
	}
	return value;
}

export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		const command = cli.program
			.command("serve", "run the rspack dev server.")
			.alias("server")
			.alias("s")
			.alias("dev");

		commonOptionsForBuildAndServe(commonOptions(command))
			.option("--hot [mode]", "enables hot module replacement")
			.option("--port <port>", "allows to specify a port to use")
			.option("--host <host>", "allows to specify a hostname to use");

		command.action(async (cliOptions: ServerOptions) => {
			setDefaultNodeEnv(cliOptions, "development");
			normalizeCommonOptions(cliOptions, "serve");
			cliOptions.hot = normalizeHotOption(cliOptions.hot);

			// Lazy import @rspack/dev-server to avoid loading it on build mode
			const { RspackDevServer } = await import("@rspack/dev-server");

			const compiler = await cli.createCompiler(cliOptions, "serve");
			if (!compiler) {
				return;
			}

			const compilers = cli.isMultipleCompiler(compiler)
				? compiler.compilers
				: [compiler];

			const possibleCompilers = compilers.filter(
				(compiler: Compiler) => compiler.options.devServer
			);

			const usedPorts: number[] = [];
			const servers: RspackDevServerType[] = [];

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
			for (const compiler of compilers) {
				const devServer = (compiler.options.devServer ??= {});

				devServer.hot = cliOptions.hot ?? devServer.hot ?? DEFAULT_SERVER_HOT;

				if (devServer.client !== false) {
					if (devServer.client === true || devServer.client == null) {
						devServer.client = {};
					}
					devServer.client = {
						overlay: {
							errors: true,
							warnings: false
						},
						...devServer.client
					};
				}
			}

			const devServerOptions = (compilerForDevServer.options.devServer ??= {});
			const { setupMiddlewares } = devServerOptions;

			const lazyCompileMiddleware =
				rspack.experiments.lazyCompilationMiddleware(compiler);

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
						warnings: false
					},
					...devServerOptions.client
				};
			}

			if (devServerOptions.port) {
				const portNumber = Number(devServerOptions.port);

				if (!Number.isNaN(portNumber)) {
					if (usedPorts.find(port => portNumber === port)) {
						throw new Error(
							"Unique ports must be specified for each devServer option in your rspack configuration. Alternatively, run only 1 devServer config using the --config-name flag to specify your desired config."
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
