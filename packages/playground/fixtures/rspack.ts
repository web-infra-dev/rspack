import path from "path";
import { Fixtures, PlaywrightTestArgs } from "@playwright/test";
import { Compiler, Configuration, rspack } from "@rspack/core";
import { RspackDevServer } from "@rspack/dev-server";
import WebpackDevServer from "webpack-dev-server";
import type { PathInfoFixtures } from "./pathInfo";
import { sleep } from "@/utils/sleep";

class RspackTestFixture {
	projectDir: string;
	compiler: Compiler;
	devServer: RspackDevServer | WebpackDevServer;
	private onDone: Array<() => void> = [];
	constructor(
		projectDir: string,
		wds: boolean,
		handleRspackConfig: (config: Configuration) => Configuration
	) {
		const configPath = path.resolve(projectDir, "rspack.config.js");
		const config = handleRspackConfig(require(configPath));
		delete require.cache[configPath];
		const compiler = rspack(config);

		this.projectDir = projectDir;
		this.compiler = compiler;
		this.compiler.hooks.done.tap("rspack_fixture", () => {
			const onDone = this.onDone;
			this.onDone = [];
			for (const item of onDone) {
				item();
			}
		});
		const DevServerConstructor = wds ? WebpackDevServer : RspackDevServer;
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

	// TODO add some plugin to watch hmr complete
	waitingForHmr(poll: () => Promise<boolean>) {
		return this.waitUntil(poll);
	}

	async waitUntil(poll: () => Promise<boolean>) {
		const maxTries = 100;
		for (let tries = 0; tries < maxTries; tries++) {
			const isSuccess = await poll();
			if (isSuccess) {
				return;
			}
			if (tries === maxTries - 1) {
				throw new Error("out of max retry time");
			}
			await sleep(200);
		}
	}
}

export type RspackOptions = {
	defaultRspackConfig: {
		handleConfig(config: Configuration): Configuration;
	};
};

export type RspackFixtures = {
	rspack: RspackTestFixture;
};

type RspackWorkerFixtures = {
	_startRspackServer: (
		testFile: string,
		tempProjectDir: string,
		handleRspackConfig: (config: Configuration) => Configuration
	) => Promise<RspackTestFixture>;
};

export const rspackFixtures = (
	wds: boolean
): Fixtures<
	RspackOptions & RspackFixtures,
	RspackWorkerFixtures,
	PlaywrightTestArgs & PathInfoFixtures
> => {
	return {
		defaultRspackConfig: [{ handleConfig: c => c }, { option: true }],
		rspack: [
			async (
				{ page, pathInfo, _startRspackServer, defaultRspackConfig },
				use
			) => {
				const rspackTest = await _startRspackServer(
					pathInfo.testFile,
					pathInfo.tempProjectDir,
					defaultRspackConfig.handleConfig
				);
				const port = rspackTest.devServer.options.port;
				await rspackTest.waitingForBuild();
				await page.goto(`http://localhost:${port}`);
				await use(rspackTest);
			},
			{
				auto: true
			}
		],

		_startRspackServer: [
			async ({}, use, { workerIndex }) => {
				let currentTestFile = "";
				let rspackTest: RspackTestFixture | null = null as any;
				await use(async function (testFile, projectDir, handleRspackConfig) {
					if (rspackTest && currentTestFile !== testFile) {
						await rspackTest.devServer.stop();
						rspackTest = null;
						currentTestFile = testFile;
					}
					if (!rspackTest) {
						const port = 8000 + workerIndex;
						rspackTest = new RspackTestFixture(projectDir, wds, function (
							config
						) {
							// rewrite port
							if (!config.devServer) {
								config.devServer = {};
							}
							config.devServer.port = port;

							// set default context
							if (!config.context) {
								config.context = projectDir;
							}

							// set default define
							(config.plugins ??= []).push(
								new rspack.DefinePlugin({
									"process.env.NODE_ENV": JSON.stringify(
										config.mode || "development"
									)
								})
							);

							return handleRspackConfig(config);
						});
						await rspackTest.devServer.start();
					}

					return rspackTest;
				});

				if (rspackTest?.projectDir) {
					await rspackTest.devServer.stop();
				}
			},
			{
				scope: "worker",
				timeout: 60000
			}
		]
	};
};
