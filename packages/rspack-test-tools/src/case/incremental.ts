import fs from "node:fs";
import path from "node:path";
import type { RspackOptions } from "@rspack/core";
import { BasicCaseCreator } from "../test/creator";
import type { ITestContext } from "../type";
import { createHotProcessor, createHotRunner } from "./hot";
import {
	createWatchInitialProcessor,
	createWatchRunner,
	createWatchStepProcessor,
	getWatchRunnerKey
} from "./watch";

type TTarget = RspackOptions["target"];

const hotCreators: Map<string, BasicCaseCreator> = new Map();

function createHotIncrementalProcessor(
	name: string,
	src: string,
	temp: string,
	target: TTarget,
	webpackCases: boolean
) {
	return createHotProcessor(name, src, temp, target, true);
}

function getHotCreator(target: TTarget, webpackCases: boolean) {
	const key = JSON.stringify({ target, webpackCases });
	if (!hotCreators.has(key)) {
		hotCreators.set(
			key,
			new BasicCaseCreator({
				clean: true,
				describe: true,
				target,
				steps: ({ name, target, src, temp, dist }) => [
					createHotIncrementalProcessor(
						name,
						src,
						temp || path.resolve(dist, "temp"),
						target as TTarget,
						webpackCases
					)
				],
				runner: {
					key: (context: ITestContext, name: string, file: string) => name,
					runner: createHotRunner
				},
				concurrent: true
			})
		);
	}
	return hotCreators.get(key)!;
}

export function createHotIncrementalCase(
	name: string,
	src: string,
	dist: string,
	temp: string,
	target: RspackOptions["target"],
	webpackCases: boolean
) {
	const creator = getHotCreator(target, webpackCases);
	creator.create(name, src, dist, temp);
}

const watchCreators: Map<string, BasicCaseCreator> = new Map();

export type WatchIncrementalOptions = {
	ignoreNotFriendlyForIncrementalWarnings?: boolean;
};

function getWatchCreator(options: WatchIncrementalOptions) {
	const key = JSON.stringify(options);
	if (!watchCreators.has(key)) {
		watchCreators.set(
			key,
			new BasicCaseCreator({
				clean: true,
				runner: {
					key: getWatchRunnerKey,
					runner: createWatchRunner
				},
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
						.filter(name => fs.statSync(path.join(src, name)).isDirectory())
						.map(name => ({ name }));

					return runs.map((run, index) =>
						index === 0
							? createWatchInitialProcessor(name, temp!, run.name, watchState, {
									incremental: true
								})
							: createWatchStepProcessor(name, temp!, run.name, watchState, {
									incremental: true
								})
					);
				},
				concurrent: true
			})
		);
	}
	return watchCreators.get(key)!;
}

export function createWatchIncrementalCase(
	name: string,
	src: string,
	dist: string,
	temp: string,
	options: WatchIncrementalOptions = {}
) {
	const creator = getWatchCreator(options);
	creator.create(name, src, dist, temp);
}
