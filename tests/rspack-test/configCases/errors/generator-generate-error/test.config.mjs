import { findOutputFiles } from "@rspack/test-tools/helper/legacy/findOutputFiles";

export default {
	findBundle(i, options) {
		const files = findOutputFiles(options, new RegExp(/\.js$/));

		return files.sort((a, _b) => (a.startsWith("main") ? 1 : 0));
	}
};
