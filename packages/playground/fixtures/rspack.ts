import path from "path";
import { Fixtures, PlaywrightTestArgs } from "@playwright/test";
import { Compiler, Configuration, createCompiler } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import type { PathInfoFixtures } from "./pathInfo";
import { sleep } from "@/utils/sleep";

class Rspack {
	projectDir: string;
	compiler: Compiler;
	devServer: RspackDevServer;
	private onDone: Array<() => void> = [];
	constructor(
		projectDir: string,
		handleRspackConfig: (config: Configuration) => Configuration
	) {
		const configPath = path.resolve(projectDir, "rspack.config.js");
		const config = handleRspackConfig(require(configPath));
		delete require.cache[configPath];
		const compiler = createCompiler(config);

		this.projectDir = projectDir;
		this.compiler = compiler;
		this.compiler.hooks.done.tap("rspack_fixture", () => {
			const onDone = this.onDone;
			this.onDone = [];
			for (const item of onDone) {
				item();
			}
		});
		this.devServer = new RspackDevServer(
			compiler.options.devServer ?? {},
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

	// TODO add some plugin to watch hmr complete
	async waitingForHmr(poll: () => Promise<boolean>) {
		const maxTries = 100;
		for (let tries = 0; tries < maxTries; tries++) {
			const isSuccess = await poll();
			if (isSuccess) {
				return;
			}
			if (tries === maxTries - 1) {
				throw new Error("outof max retry time");
			}
			await sleep(50);
		}
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

type RspackWorkerFixtures = {
	_startRspackServer: (
		testFile: string,
		tempProjectDir: string,
		handleRspackConfig: (config: Configuration) => Configuration
	) => Promise<Rspack>;
};

export const rspackFixtures: Fixtures<
	RspackOptions & RspackFixtures,
	RspackWorkerFixtures,
	PlaywrightTestArgs & PathInfoFixtures
> = {
	defaultRspackConfig: [{ handleConfig: c => c }, { option: true }],
	rspack: [
		async (
			{ page, pathInfo, _startRspackServer, defaultRspackConfig },
			use
		) => {
			const rspack = await _startRspackServer(
				pathInfo.testFile,
				pathInfo.tempProjectDir,
				defaultRspackConfig.handleConfig
			);
			const port = rspack.devServer.options.port;
			await rspack.waitingForBuild();
			await page.goto(`http://localhost:${port}`);
			await use(rspack);
		},
		{
			auto: true
		}
	],

	_startRspackServer: [
		async ({}, use, { workerIndex }) => {
			let currentTestFile = "";
			let rspack: Rspack | null = null as any;
			await use(async function (testFile, projectDir, handleRspackConfig) {
				if (rspack && currentTestFile !== testFile) {
					await rspack.devServer.stop();
					rspack = null;
					currentTestFile = testFile;
				}
				if (!rspack) {
					const port = 8000 + workerIndex;
					rspack = new Rspack(projectDir, function (config) {
						if (!config.devServer) {
							config.devServer = {};
						}
						config.devServer.port = port;

						return handleRspackConfig(config);
					});
					await rspack.devServer.start();
				}

				return rspack;
			});

			if (rspack?.projectDir) {
				await rspack.devServer.stop();
			}
		},
		{
			scope: "worker",
			timeout: 60000
		}
	]
};
