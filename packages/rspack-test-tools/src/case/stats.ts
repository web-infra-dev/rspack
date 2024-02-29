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

	beforeEach(async () => {
		rimraf.sync(dist);
		fs.mkdirSync(dist, { recursive: true });
		await tester.prepare();
	});

	it(`should print correct stats for ${name}`, async () => {
		await tester.compile();
		await tester.check({
			it,
			beforeEach,
			afterEach
		});
	}, 30000);

	afterEach(async () => {
		await tester.resume();
	});
}
