import fs from "node:fs";
import path from "node:path";

import { HotNewIncrementalProcessor } from "../processor/hot-new-incremental";
import { WatchProcessor, WatchStepProcessor } from "../processor/watch";
import { HotRunnerFactory, WatchRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType, type TCompilerOptions } from "../type";

type TTarget = TCompilerOptions<ECompilerType.Rspack>["target"];

const hotCreators: Map<
	string,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

function getHotCreator(target: TTarget, webpackCases: boolean) {
	const key = JSON.stringify({ target, webpackCases });
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
						webpackCases
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
	webpackCases: boolean
) {
	const creator = getHotCreator(target, webpackCases);
	creator.create(name, src, dist);
}

const watchCreators: Map<
	string,
	BasicCaseCreator<ECompilerType.Rspack>
> = new Map();

export type WatchNewIncrementalOptions = {
	ignoreNotFriendlyForIncrementalWarnings?: boolean;
};

function getWatchCreator(options: WatchNewIncrementalOptions) {
	const key = JSON.stringify(options);
	if (!watchCreators.has(key)) {
		watchCreators.set(
			key,
			new BasicCaseCreator({
				clean: true,
				runner: WatchRunnerFactory,
				description: (name, index) => {
					return index === 0
						? `${name} should compile`
						: `should compile step ${index}`;
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
										defaultOptions(index, context) {
											return {
												experiments: {
													incremental: "advance"
												},
												ignoreWarnings:
													options.ignoreNotFriendlyForIncrementalWarnings
														? [/is not friendly for incremental/]
														: undefined
											};
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
			})
		);
	}
	return watchCreators.get(key)!;
}

export function createWatchNewIncrementalCase(
	name: string,
	src: string,
	dist: string,
	temp: string,
	options: WatchNewIncrementalOptions = {}
) {
	const creator = getWatchCreator(options);
	creator.create(name, src, dist, temp);
}
