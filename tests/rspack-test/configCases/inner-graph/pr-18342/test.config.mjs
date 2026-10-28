import { findOutputFiles } from "@rspack/test-tools/helper/legacy/findOutputFiles";

export default {
	findBundle(_, options) {
		const files = findOutputFiles(options, /^entry/);
		return files;
	}
};
