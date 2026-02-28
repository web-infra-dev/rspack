import fs from "fs";
import Worker from "worker-rspack-loader!./worker.js";

it("should work", () => {
	Worker;
	let file = fs.readFileSync(__dirname + "/bundle0.worker.js", "utf-8");
	expect(file).toContain('module.exports = "str"');
});
