const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { execFileSync } = require("node:child_process");

module.exports = {
	findBundle() { return []; },
	afterExecute(options) {
		fs.cpSync(path.join(__dirname, "../request-order/fixtures"), path.join(options.output.path, "fixtures"), { recursive: true });
		const url = pathToFileURL(path.join(options.output.path, "main.mjs")).href;
		const order = JSON.parse(execFileSync(process.execPath, ["--input-type=module", "-e", `
			await import(${JSON.stringify(url)});
			console.log(JSON.stringify(globalThis.order));
		`], { encoding: "utf8" }));
		expect(order).toEqual(["a", "b", "shared", "c"]);
	}
};
