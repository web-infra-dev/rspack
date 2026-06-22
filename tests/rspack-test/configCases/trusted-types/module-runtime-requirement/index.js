const fs = require("fs");
const path = require("path");


function createWorker() {
	new Worker(new URL("./worker.js", import.meta.url), {
		type: "module",
		name: "test-worker"
	});
}

createWorker;

it("should generate correct new Worker statement", async () => {
	const content = fs.readFileSync(path.resolve(path.dirname(__filename), './test-worker.js'), "utf-8");
	expect(content).toContain(`this is worker`);
	if (content.includes("__rspack_context")) {
		expect(content).toContain(`(__unused_rspack_module, __unused_rspack_exports, __rspack_context)`);
		expect(content).toContain(`eval(__rspack_context.ts(`);
	} else {
		expect(content).toContain(`(__unused_rspack_module, __unused_rspack_exports, __webpack_require__)`);
		expect(content).toContain(`eval(__webpack_require__.ts(`);
	}
});
