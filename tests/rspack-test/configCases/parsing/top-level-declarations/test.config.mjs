import { createFakeWorker } from "@rspack/test-tools/helper/legacy/createFakeWorker";
let outputDirectory;

export default {
	moduleScope(scope) {
		const FakeWorker = createFakeWorker({ expect }, {
			outputDirectory
		});

		scope.AudioContext = class AudioContext {
			constructor() {
				this.audioWorklet = {
					addModule: (url) => Promise.resolve(FakeWorker.bind(null, url))
				};
			}
		};
	},
	findBundle(i, options) {
		outputDirectory = options.output.path;
		return ["main.js"];
	}
};
