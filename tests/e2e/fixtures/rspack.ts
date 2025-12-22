import path from "node:path";
import type { Fixtures, PlaywrightTestArgs } from "@playwright/test";
import {
	type Compiler,
	type Configuration,
	rspack,
	type RspackOptions as RspackConfig,
} from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import type { PathInfoFixtures } from "./pathInfo";

class Rspack {
	private config: RspackConfig;
	projectDir: string;
	outDir: string;
	compiler: Compiler;
	devServer: RspackDevServer;
	private onDone: Array<() => void> = [];
	constructor(
		projectDir: string,
		handleRspackConfig: (config: Configuration) => Configuration
	) {
		const configPath = path.resolve(projectDir, "rspack.config.js");
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

		return new Promise<void>(resolve => {
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
		this.compiler = compiler;
		this.compiler.hooks.done.tap("rspack_fixture", () => {
			const onDone = this.onDone;
			this.onDone = [];
			for (const item of onDone) {
				item();
			}
		});
		const DevServerConstructor = RspackDevServer;
		if (compiler.options.lazyCompilation) {
			const middleware = rspack.lazyCompilationMiddleware(compiler);
			compiler.options.devServer ??= {};
			const setupMiddlewares = compiler.options.devServer.setupMiddlewares;
			compiler.options.devServer.setupMiddlewares = (middlewares, server) => {
				const old = setupMiddlewares
					? setupMiddlewares(middlewares, server)
					: middlewares;
				return [middleware, ...old];
			};
		}
		this.devServer = new DevServerConstructor(
			compiler.options.devServer ?? ({} as any),
			compiler
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

export type RspackFixtures = Fixtures<
	RspackOptions & { rspack: Rspack },
	{},
	PlaywrightTestArgs & PathInfoFixtures
>;

export const rspackFixtures = (): RspackFixtures => {
	return {
		rspackConfig: [{ handleConfig: c => c, basePort: 8000 }, { option: true }],
		rspack: [
			async ({ page, pathInfo, rspackConfig }, use, { workerIndex }) => {
				const { tempProjectDir } = pathInfo;
				const port = rspackConfig.basePort + workerIndex;
				const rspack = new Rspack(tempProjectDir, config => {
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
					config.output.path = path.resolve(tempProjectDir, "dist");

					return rspackConfig.handleConfig(config);
				});
				await rspack.start();

				await page.goto(`http://localhost:${port}`);

				await use(rspack);

				await rspack.stop();
			},
			{ auto: true }
		]
	};
};
