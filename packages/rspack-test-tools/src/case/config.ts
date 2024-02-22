import { Tester } from "../test/tester";
import rimraf from "rimraf";
import { RspackConfigProcessor } from "../processor/config";
import fs from "fs-extra";
import path from "path";

export function createConfigCase(name: string, src: string, dist: string) {
	const testConfigFile = path.join(src, "test.config.js");
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackConfigProcessor({
				name,
				testConfig: fs.existsSync(testConfigFile) ? require(testConfigFile) : {}
			})
		]
	});

	describe(name, () => {
		rimraf.sync(dist);
		fs.mkdirSync(dist, { recursive: true });

		beforeAll(async () => {
			await tester.prepare();
		});

		it(`${name} should compile`, async () => {
			await tester.compile();
			await tester.check(env);
		}, 30000);

		afterAll(async () => {
			await tester.resume();
		});

		const env = Tester.createTestEnv();
	});
}
