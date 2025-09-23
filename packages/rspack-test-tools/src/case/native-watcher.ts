import fs from "node:fs";
import path from "node:path";

import { WatchRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { createWatchInitialProcessor, createWatchStepProcessor } from "./watch";

const creator = new BasicCaseCreator({
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
			.filter(name => fs.statSync(path.join(src, name)).isDirectory())
			.map(name => ({ name }));

		return runs.map((run, index) =>
			index === 0
				? createWatchInitialProcessor(name, temp!, run.name, watchState, {
						nativeWatcher: true
					})
				: createWatchStepProcessor(name, temp!, run.name, watchState, {
						nativeWatcher: true
					})
		);
	},
	concurrent: true
});

export function createNativeWatcher(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	creator.create(name, src, dist, temp);
}
