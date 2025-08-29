import { DiagnosticProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType } from "../type";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new DiagnosticProcessor({
			name,
			snapshot: "./stats.err",
			snapshotErrors: "./raw-error.err",
			snapshotWarning: "./raw-warning.err",
			configFiles: ["rspack.config.js", "webpack.config.js"],
			compilerType: ECompilerType.Rspack,
			format: (output: string) => {
				return output;
			}
		})
	],
	concurrent: true
});

export function createDiagnosticCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
