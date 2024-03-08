import { ECompilerType, ITester, TTestConfig } from "../type";
import { RspackHashProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";
class HashCaseCreator<T extends ECompilerType> extends BasicCaseCreator<T> {
	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		it(`should print correct hash for ${name}`, async () => {
			await tester.prepare();
			await tester.compile();
			await tester.check(this.createEnv(testConfig));
			await tester.resume();
		}, 30000);
	}
}

const creator = new HashCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new RspackHashProcessor({
			name
		})
	]
});

export function createHashCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
