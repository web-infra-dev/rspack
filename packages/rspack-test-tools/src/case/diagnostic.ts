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
			configFiles: ["rspack.config.js", "webpack.config.js"],
			compilerType: ECompilerType.Rspack,
			format: (output: string) => {
				// TODO: change to stats.errorStack
				// TODO: add `errorStack: false`
				return output.replace(/(?:\s|│)*(at .*)(\s|│)*/g, "");
			}
		})
	]
});

export function createDiagnosticCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
