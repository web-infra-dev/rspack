import { Tester } from "../test/tester";
import rimraf from "rimraf";
import fs from "fs";
import path from "path";
import { RspackNormalProcessor } from "../processor/normal";
import { ECompilerType, TCompilerOptions } from "..";

export function createNormalCase(
	name: string,
	src: string,
	dist: string,
	compilerOptions: TCompilerOptions<ECompilerType.Rspack> = {}
) {
	const testConfigFile = path.join(src, "test.config.js");
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackNormalProcessor({
				name,
				compilerOptions,
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

		const env = Tester.createLazyTestEnv();
	});
}
