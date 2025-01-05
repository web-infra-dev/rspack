import { join } from 'node:path';
import { rspack } from "@rspack/core";
import { beforeAll, bench, describe } from "vitest";
import { breakpoint } from './loaders/noop';

beforeAll(() => {
	const use: string[] = [];
	const loader = require.resolve("./loaders/noop");
	for (let i = 0; i < 100; i++) {
		use.push(loader);
	}

	return new Promise<void>((resolve, reject) => {

		class LoaderBenchPlguin {
			succeedModuleCallback: () => void;
		
			apply(compiler) {
				const pluginName = this.constructor.name;
		
				compiler.hooks.compilation.tap(pluginName, compilation => {
					compilation.hooks.buildModule.tap(pluginName, () => {
						resolve();
					});
				});
			}
		}

		rspack(
			{
				entry: join(__dirname, "fixtures/vanilla-js/one.js"),
				mode: "production",
				module: {
					rules: [
						{
						  use
						},
					],
				},
				plugins: [
					new LoaderBenchPlguin(),
				]
			},
			(err, stats) => {
				if (err) {
					reject(err);
				}
				if (stats?.hasErrors()) {
					reject(new Error(stats.toString({})));
				}
				reject(new Error("Build exited prematurely"));
			}
		)
	});
});

describe("Noop loader", () => {
	bench("Rust dispatch javascript loader", async () => {
		breakpoint.next();
		await breakpoint.paused();
	});
});
