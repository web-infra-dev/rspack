import path from "node:path";
import type { Fixtures, PlaywrightTestArgs } from "@playwright/test";
import {
	type Compiler,
	type Configuration,
	rspack,
	type RspackOptions as RspackConfig
} from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import WebpackDevServer from "webpack-dev-server";
import type { PathInfoFixtures } from "./pathInfo";

class Rspack {
	private wds: boolean;
	private config: RspackConfig;
	projectDir: string;
	compiler: Compiler;
	devServer: RspackDevServer | WebpackDevServer;
	private onDone: Array<() => void> = [];
	constructor(
		projectDir: string,
		wds: boolean,
		handleRspackConfig: (config: Configuration) => Configuration
	) {
		this.wds = wds;

		const configPath = path.resolve(projectDir, "rspack.config.js");
		this.config = handleRspackConfig(require(configPath));
		delete require.cache[configPath];
		const compiler = rspack(this.config);

		this.projectDir = projectDir;
		this.compiler = compiler;
		this.compiler.hooks.done.tap("rspack_fixture", () => {
			const onDone = this.onDone;
			this.onDone = [];
			for (const item of onDone) {
				item();
			}
		});
		const DevServerConstructor = this.wds ? WebpackDevServer : RspackDevServer;
		this.devServer = new DevServerConstructor(
			compiler.options.devServer ?? ({} as any),
			compiler
		);
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

	async reboot() {
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

		const compiler = rspack(this.config);
		compiler.hooks.done.tap("rspack_fixture", () => {
			const onDone = this.onDone;
			this.onDone = [];
			for (const item of onDone) {
				item();
			}
		});
		const DevServerConstructor = this.wds ? WebpackDevServer : RspackDevServer;
		this.devServer = new DevServerConstructor(
			compiler.options.devServer ?? ({} as any),
			compiler
		);
		this.compiler = compiler;

		await this.devServer.start();
		await this.waitingForBuild();
	}
}

export type RspackOptions = {
	defaultRspackConfig: {
		handleConfig(config: Configuration): Configuration;
	};
};

export type RspackFixtures = {
	rspack: Rspack;
};

export const rspackFixtures = (
	wds: boolean
): Fixtures<
	RspackOptions & RspackFixtures,
	PlaywrightTestArgs & PathInfoFixtures
> => {
	return {
		defaultRspackConfig: [{ handleConfig: c => c }, { option: true }],
		rspack: [
			async ({ page, pathInfo, defaultRspackConfig }, use, { workerIndex }) => {
				const { tempProjectDir } = pathInfo;
				const port = 8000 + workerIndex;
				const rspack = new Rspack(tempProjectDir, wds, config => {
					// rewrite port
					if (!config.devServer) {
						config.devServer = {};
					}
					config.devServer.port = port;

					// set default context
					if (!config.context) {
						config.context = tempProjectDir;
					}

					return defaultRspackConfig.handleConfig(config);
				});
				await rspack.devServer.start();

				await rspack.waitingForBuild();
				await page.goto(`http://localhost:${port}`);

				await use(rspack);

				await rspack.devServer.stop();
			},
			{
				auto: true
			}
		]
	};
};
