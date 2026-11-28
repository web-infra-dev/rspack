import fs from "node:fs";
import path from "node:path";

const cssRequests = [];
let initialCssUrl;

export default {
	findBundle(index, options) {
		const files = fs.readdirSync(options.output.path);
		const initialCssFiles = files.filter(
			file => file.endsWith(".css") && !file.endsWith(".chunk.css")
		);
		expect(initialCssFiles).toHaveLength(2);
		const sharedCssFile = initialCssFiles.find(file => file.startsWith("shared."));
		const initialOnlyCssFile = initialCssFiles.find(file =>
			file.startsWith("initial-only.")
		);
		expect(sharedCssFile).toBeTruthy();
		expect(initialOnlyCssFile).toBeTruthy();

		const runtime = fs.readFileSync(
			path.join(options.output.path, "runtime~index.js"),
			"utf-8"
		);
		const contentHash = file => file.split(".")[1];
		expect(runtime).toContain(contentHash(sharedCssFile));
		expect(runtime).not.toContain(contentHash(initialOnlyCssFile));

		initialCssUrl = new URL(sharedCssFile, options.output.publicPath).href;
		return [...initialCssFiles, "runtime~index.js", "index.js"];
	},
	resourceLoader(url) {
		if (url.endsWith(".css")) cssRequests.push(url);
	},
	moduleScope(scope) {
		scope.getCssRequests = () => cssRequests;
		scope.getInitialCssUrl = () => initialCssUrl;
	},
};
