import { Tester } from "../test/tester";
import rimraf from "rimraf";
import fs from "fs";
import path from "path";
import {
	RspackWatchProcessor,
	RspackWatchStepProcessor
} from "../processor/watch";

export function createWatchCase(
	name: string,
	src: string,
	dist: string,
	temp: string
) {
	const testConfigFile = path.join(src, "test.config.js");
	const runs = fs
		.readdirSync(src)
		.sort()
		.filter(name => {
			return fs.statSync(path.join(src, name)).isDirectory();
		})
		.map(name => ({ name }));

	const testConfig = fs.existsSync(testConfigFile)
		? require(testConfigFile)
		: {};
	const tester = new Tester({
		name,
		src,
		dist,
		steps: runs.map((run, index) =>
			index === 0
				? new RspackWatchProcessor({
						name,
						stepName: run.name,
						tempDir: temp,
						testConfig
				  })
				: new RspackWatchStepProcessor({
						name,
						stepName: run.name,
						tempDir: temp,
						testConfig
				  })
		)
	});

	if (
		Tester.isSkipped({
			casePath: src,
			name
		})
	) {
		describe.skip(name, () => {
			it("filtered", () => {});
		});
		return;
	}

	rimraf.sync(dist);
	rimraf.sync(temp);
	fs.mkdirSync(dist, { recursive: true });
	fs.mkdirSync(temp, { recursive: true });

	beforeAll(async () => {
		await tester.prepare();
	});

	for (const index of runs.keys()) {
		it(
			index === 0 ? `${name} should compile` : "should compile the next step",
			async () => {
				await tester.compile();
				await tester.check(env);
				tester.next();
			},
			30000
		);
		const env = Tester.createLazyTestEnv();
	}

	afterAll(async () => {
		await tester.resume();
	});
}
