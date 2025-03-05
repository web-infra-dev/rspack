import fs from "node:fs";
import path from "node:path";

import { WatchProcessor, WatchStepProcessor } from "../processor/watch";
import { WatchRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const creator = new BasicCaseCreator({
	clean: true,
	runner: WatchRunnerFactory,
	description: (name, index) => {
		return index === 0
			? `${name} should compile`
			: `should compile the next step ${index}`;
	},
	describe: false,
	steps: ({ name, src, temp }) => {
		const watchState = {};
		const runs = fs
			.readdirSync(src)
			.sort()
			.filter(name => {
				return fs.statSync(path.join(src, name)).isDirectory();
			})
			.map(name => ({ name }));

		return runs.map((run, index) =>
			index === 0
				? new WatchProcessor(
						{
							name,
							stepName: run.name,
							tempDir: temp!,
							runable: true,
							compilerType: ECompilerType.Rspack,
							configFiles: ["rspack.config.js", "webpack.config.js"]
						},
						watchState
					)
				: new WatchStepProcessor(
						{
							name,
							stepName: run.name,
							tempDir: temp!,
							runable: true,
							compilerType: ECompilerType.Rspack,
							configFiles: ["rspack.config.js", "webpack.config.js"]
						},
						watchState
					)
		);
	},
	concurrent: true
});

export function createWatchCase(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	creator.create(name, src, dist, temp);
}
