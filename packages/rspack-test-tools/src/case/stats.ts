import { Tester } from "../test/tester";
import rimraf from "rimraf";
import fs from "fs";
import path from "path";
import { RspackStatsProcessor } from "../processor/stats";

export function createStatsCase(name: string, src: string, dist: string) {
	const testConfigFile = path.join(src, "test.config.js");
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackStatsProcessor({
				name,
				testConfig: fs.existsSync(testConfigFile) ? require(testConfigFile) : {}
			})
		]
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
	fs.mkdirSync(dist, { recursive: true });

	beforeAll(async () => {
		await tester.prepare();
	});

	it(`should print correct stats for ${name}`, async () => {
		await tester.compile();
		await tester.check(env);
	}, 30000);

	afterAll(async () => {
		await tester.resume();
	});

	const env = Tester.createTestEnv();
}
