import { SnapshotProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";
import {
	ECompilerType,
	type ITestContext,
	type TCompilerOptions
} from "../type";
import { defaultOptions } from "./builtin";

function overrideOptions(
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>
) {
	options.target = options.target || ["web", "es2022"];
	options.optimization ??= {};
	options.optimization.providedExports = true;
	options.optimization.innerGraph = true;
	options.optimization.usedExports = true;

	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	description(name, step) {
		return `${name} with newTreeshaking should match snapshot`;
	},
	steps: ({ name }) => [
		new SnapshotProcessor({
			name,
			snapshot: "treeshaking.snap.txt",
			compilerType: ECompilerType.Rspack,
			runable: false,
			defaultOptions,
			overrideOptions
		})
	]
});

export function createTreeShakingCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
