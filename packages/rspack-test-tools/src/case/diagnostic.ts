import { RspackDiagnosticProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		new RspackDiagnosticProcessor({
			name
		})
	]
});

export function createDiagnosticCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
