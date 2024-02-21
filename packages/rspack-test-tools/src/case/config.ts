import { Tester } from "../test/tester";
import rimraf from "rimraf";
import { RspackConfigProcessor } from "../processor/config";

export function createConfigCase(name: string, src: string, dist: string) {
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackConfigProcessor({
				name
			})
		]
	});

	describe(name, () => {
		beforeAll(async () => {
			rimraf.sync(dist);
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
