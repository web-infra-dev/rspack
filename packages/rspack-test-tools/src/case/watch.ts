import fs from "fs";
import path from "path";
import {
	RspackWatchProcessor,
	RspackWatchStepProcessor
} from "../processor/watch";
import { BasicCaseCreator } from "../test/creator";
import { WatchRunnerFactory } from "../runner";

const creator = new BasicCaseCreator({
	clean: true,
	runner: WatchRunnerFactory,
	description: (name, index) => {
		return index === 0
			? `${name} should compile`
			: "should compile the next step";
	},
	describe: false,
	steps: ({ name, src, temp }) => {
		const runs = fs
			.readdirSync(src)
			.sort()
			.filter(name => {
				return fs.statSync(path.join(src, name)).isDirectory();
			})
			.map(name => ({ name }));

		return runs.map((run, index) =>
			index === 0
				? new RspackWatchProcessor({
						name,
						stepName: run.name,
						tempDir: temp!,
						runable: true
				  })
				: new RspackWatchStepProcessor({
						name,
						stepName: run.name,
						tempDir: temp!,
						runable: true
				  })
		);
	}
});

export function createWatchCase(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	creator.create(name, src, dist, temp);
}
