"use strict";

const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle() {
		return ["basic.js"];
	},
	afterExecute(options) {
		// Sort: readdirSync order is filesystem-dependent (differs on Bun).
		const files = fs
			.readdirSync(options.output.path)
			.filter((item) => !/stats/.test(item))
			.sort();

		expect(files).toEqual([
			"basic.js",
			"common.css",
			"common.js",
			"five.css",
			"four.css",
			"one.css",
			"six.css",
			"six.js",
			"three.css",
			"two.css"
		]);

		for (const file of files.filter((item) => /\.css/.test(item))) {
			expect(
				fs.readFileSync(path.join(options.output.path, file), "utf8")
			).toMatchFileSnapshotSync(path.join(__dirname, "__snapshots__", file));
		}
	}
};
