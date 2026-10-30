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
		expect(initialCssFiles).toHaveLength(1);
		initialCssUrl = new URL(initialCssFiles[0], options.output.publicPath).href;
		return [initialCssFiles[0], "runtime~index.js", "index.js"];
	},
	resourceLoader(url) {
		if (url.endsWith(".css")) cssRequests.push(url);
	},
	moduleScope(scope) {
		scope.getCssRequests = () => cssRequests;
		scope.getInitialCssUrl = () => initialCssUrl;
	},
};
