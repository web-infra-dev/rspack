import { Compiler, DevServer } from "@rspack/core";
import type { RspackDevServer as RspackDevServerType } from "@rspack/dev-server";

import type { RspackCLI } from "../rspack-cli";
import { RspackCommand } from "../types";
import {
	commonOptions,
	ensureEnvObject,
	setBuiltinEnvArg
} from "../utils/options";

export class ServeCommand implements RspackCommand {
	async apply(cli: RspackCLI): Promise<void> {
		cli.program.command(
			["serve", "server", "s", "dev"],
			"run the rspack dev server.",
			commonOptions,
			async options => {
				setBuiltinEnvArg(ensureEnvObject(options), "SERVE", true);
				const rspackOptions = {
					...options,
					argv: {
						...options
					}
				};
				/**
				 * webpack-dev-server will set `process.env.WEBPACK_SERVE` to true
				 * when its module is imported, so we have to lazy load the package
				 * to make sure the envvar is not set on build mode.
				 * when run in serve mode, we have to load the package before config
				 * module is imported so that the envvar `process.env.WEBPACK_SERVE`
				 * got in config module could be `true`.
				 * related issue: https://github.com/web-infra-dev/rspack/issues/6359
				 */
				const { RspackDevServer } = await import("@rspack/dev-server");

				const compiler = await cli.createCompiler(rspackOptions, "serve");
				if (!compiler) return;
				const compilers = cli.isMultipleCompiler(compiler)
					? compiler.compilers
					: [compiler];
				const possibleCompilers = compilers.filter(
					(compiler: Compiler) => compiler.options.devServer
				);

				const usedPorts: number[] = [];
				const servers: RspackDevServerType[] = [];

				/**
				 * Webpack uses an Array of compilerForDevServer,
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
					devServer.hot ??= true;
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

				const result = (compilerForDevServer.options.devServer ??= {});
				/**
				 * Enable this to tell Rspack that we need to enable React Refresh by default
				 */
				result.hot ??= true;
				if (result.client !== false) {
					if (result.client === true || result.client == null) {
						result.client = {};
					}
					result.client = {
						overlay: {
							errors: true,
							warnings: false
						},
						...result.client
					};
				}

				const devServerOptions = result as DevServer;
				if (devServerOptions.port) {
					const portNumber = Number(devServerOptions.port);

					if (usedPorts.find(port => portNumber === port)) {
						throw new Error(
							"Unique ports must be specified for each devServer option in your rspack configuration. Alternatively, run only 1 devServer config using the --config-name flag to specify your desired config."
						);
					}

					usedPorts.push(portNumber);
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
			}
		);
	}
}
