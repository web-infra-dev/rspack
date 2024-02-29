import { Tester } from "../test/tester";
import rimraf from "rimraf";
import { RspackDiagnosticProcessor } from "../processor/diagnostic";
import fs from "fs";

export function createDiagnosticCase(
	name: string,
	src: string,
	dist: string,
	root: string
) {
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [
			new RspackDiagnosticProcessor({
				name,
				root
			})
		]
	});

	describe(name, () => {
		rimraf.sync(dist);
		fs.mkdirSync(dist, { recursive: true });
		let cwd = process.cwd();

		beforeAll(async () => {
			await tester.prepare();
		});

		it(`${name} should compile`, async () => {
			await tester.compile();
			await tester.check({
				it,
				beforeEach,
				afterEach
			});
		}, 30000);

		afterAll(async () => {
			await tester.resume();
		});
	});
}
