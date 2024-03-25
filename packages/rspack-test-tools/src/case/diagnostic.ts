import path from "path";
import { RspackDiagnosticProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: true,
	steps: ({ name }) => [
		new RspackDiagnosticProcessor({
			name,
			root: path.resolve(__dirname, "../../../rspack")
		})
	]
});

export function createDiagnosticCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
