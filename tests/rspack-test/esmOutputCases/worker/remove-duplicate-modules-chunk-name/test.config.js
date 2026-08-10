const fs = require("fs");
const path = require("path");

module.exports = {
	snapshotFileFilter(file) {
		return (
			file.endsWith("main.mjs") ||
			file.endsWith("lib_js.mjs") ||
			file.endsWith("named-worker-a.mjs") ||
			file.endsWith("named-worker-b.mjs")
		);
	},
	snapshotContent(content) {
		return content.replace(/[ \t]+$/gm, "");
	},
	afterExecute(options) {
		const outputFiles = fs
			.readdirSync(options.output.path)
			.filter(file => file.endsWith(".mjs"));
		const filesWithWorkerModule = outputFiles.filter(file =>
			fs
				.readFileSync(path.join(options.output.path, file), "utf-8")
				.includes("namedWorkerModuleMarker")
		);

		expect(outputFiles).toContain("named-worker-a.mjs");
		expect(outputFiles).toContain("named-worker-b.mjs");
		expect(filesWithWorkerModule).toHaveLength(1);
	}
};
