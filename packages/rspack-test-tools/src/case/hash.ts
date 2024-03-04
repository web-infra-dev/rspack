import { Tester } from "../test/tester";
import rimraf from "rimraf";
import fs from "fs";
import path from "path";
import { RspackHashProcessor } from "../processor/hash";

export function createHashCase(name: string, src: string, dist: string) {
	const testConfigFile = path.join(src, "test.config.js");
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackHashProcessor({
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

	it(`should print correct hash for ${name}`, async () => {
		rimraf.sync(dist);
		fs.mkdirSync(dist, { recursive: true });

		await tester.prepare();
		await tester.compile();
		await tester.check(Tester.createTestEnv());
		await tester.resume();
	}, 30000);
}
