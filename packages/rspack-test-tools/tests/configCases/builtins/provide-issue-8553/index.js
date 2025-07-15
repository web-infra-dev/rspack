import Worker from "worker-rspack-loader!./worker.js";
import fs from "fs";

it("should work", () => {
	Worker;
	const file = fs.readFileSync(__dirname + "/bundle0.worker.js", "utf-8");
	expect(file).toContain('module.exports = "str"');
});
