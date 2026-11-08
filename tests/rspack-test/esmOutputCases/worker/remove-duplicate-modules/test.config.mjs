import fs from "node:fs";
import path from "node:path";

export default {
	snapshotContent(content) {
		return content.replace(/[ \t]+$/gm, "");
	},
	afterExecute(options) {
		const filesWithWorkerModule = fs
			.readdirSync(options.output.path)
			.filter(file => file.endsWith(".mjs"))
			.filter(file =>
				fs
					.readFileSync(path.join(options.output.path, file), "utf-8")
					.includes("workerModuleMarker")
			);

		expect(filesWithWorkerModule).toHaveLength(1);
	}
};
