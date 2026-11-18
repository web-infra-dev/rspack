import { findOutputFiles } from "@rspack/test-tools/helper/legacy/findOutputFiles";

export default {
	findBundle: function (i, options) {
		const chunks = findOutputFiles(options, /^chunk\./);
		expect(chunks).toHaveLength(8);
		for (const chunk of chunks) {
			expect(chunk).toMatch(/^chunk\..+\.[a-f]\.js$/);
		}
		return "./bundle0.js";
	}
};
