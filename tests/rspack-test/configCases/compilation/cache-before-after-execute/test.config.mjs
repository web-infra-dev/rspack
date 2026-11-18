import fs from "node:fs";
import path from "node:path";

export default {
	afterExecute(options) {
		expect(options.cache).toBe(false);
		const fixture = options.resolve.alias["temporary-fixture"];
		expect(fs.existsSync(fixture)).toBe(true);
		fs.rmSync(path.dirname(fixture), { recursive: true });
	}
};
