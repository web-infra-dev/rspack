import fs from "fs";
import path from "path";
import {
	RspackWatchProcessor,
	RspackWatchStepProcessor
} from "../processor/watch";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType, ITester, TTestConfig } from "../type";
import { WatchRunnerFactory } from "../runner";

class WatchCaseCreator<T extends ECompilerType> extends BasicCaseCreator<T> {
	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		beforeAll(async () => {
			await tester.prepare();
		});

		for (let index = 0; index < tester.total; index++) {
			it(
				index === 0 ? `${name} should compile` : "should compile the next step",
				async () => {
					await tester.compile();
					await tester.check(env);
					tester.next();
				},
				5000
			);
			const env = this.createEnv(testConfig);
		}

		afterAll(async () => {
			await tester.resume();
		});
	}
}

const creator = new WatchCaseCreator({
	clean: true,
	runner: WatchRunnerFactory,
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
