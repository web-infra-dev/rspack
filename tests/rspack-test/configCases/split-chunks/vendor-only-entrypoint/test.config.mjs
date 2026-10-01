import { readdirSync } from "node:fs";

export default {
	findBundle: function (i, options) {
		const files = readdirSync(options.output.path).filter(file => file.endsWith(".js"));
		expect(files.sort()).toEqual([
			"runtime~vendor.js",
			"vendor.js"
		]);
		return ["runtime~vendor.js", "vendor.js"];
	}
};
