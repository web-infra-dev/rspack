import fs from "node:fs";
import path from "node:path";
import { findOutputFiles } from "@rspack/test-tools/helper/legacy/findOutputFiles";

const chunks = [];

export default {
	findBundle: function (i, options) {
		const chunk = findOutputFiles(options, new RegExp(`^chunk${i}\\.`))[0];
		chunks.push({
			hash: /\.([^.]+)\.js$/.exec(chunk)[1],
			content: fs.readFileSync(path.join(options.output.path, chunk), "utf-8")
		});
		return `./bundle${i}.js`;
	},
	afterExecute: () => {
		const [first, second] = chunks;
		expect(first.content).not.toBe(second.content);
		expect(first.hash).not.toBe(second.hash);
	}
};
