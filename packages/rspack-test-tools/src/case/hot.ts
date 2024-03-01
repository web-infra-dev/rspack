import { Tester } from "../test/tester";
import rimraf from "rimraf";
import fs from "fs";
import { RspackHotProcessor } from "../processor/hot";
import { ECompilerType, TCompilerOptions } from "../type";

export function createHotCase(
	name: string,
	src: string,
	dist: string,
	target: TCompilerOptions<ECompilerType.Rspack>["target"]
) {
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackHotProcessor({
				target,
				name
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
