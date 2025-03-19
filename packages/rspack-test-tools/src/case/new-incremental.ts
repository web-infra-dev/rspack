import fs from "node:fs";
import path from "node:path";

import { HotNewIncrementalProcessor } from "../processor/hot-new-incremental";
import { WatchProcessor, WatchStepProcessor } from "../processor/watch";
import { HotRunnerFactory, WatchRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import {
	ECompilerType,
	type EDocumentType,
	type TCompilerOptions
} from "../type";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const hotCreators: Map<
	string,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

function getHotCreator(target: TTarget, documentType: EDocumentType) {
	const key = JSON.stringify({ target, documentType });
	if (!hotCreators.has(key)) {
		hotCreators.set(
			key,
			new BasicCaseCreator({
				clean: true,
				describe: true,
				target,
				steps: ({ name, target }) => [
					new HotNewIncrementalProcessor({
						name,
						target: target as TTarget,
						compilerType: ECompilerType.Rspack,
						configFiles: ["rspack.config.js", "webpack.config.js"],
						documentType
					})
				],
				runner: HotRunnerFactory,
				concurrent: true
			})
		);
	}
	return hotCreators.get(key)!;
}

export function createHotNewIncrementalCase(
	name: string,
	src: string,
	dist: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"],
	documentType: EDocumentType
) {
	const creator = getHotCreator(target, documentType);
	creator.create(name, src, dist);
}

const watchCreator = new BasicCaseCreator({
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
							configFiles: ["rspack.config.js", "webpack.config.js"],
							experiments: {
								incremental: true
							}
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

export function createWatchNewIncrementalCase(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	watchCreator.create(name, src, dist, temp);
}
