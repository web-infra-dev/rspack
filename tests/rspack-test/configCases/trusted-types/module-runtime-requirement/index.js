const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");


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
	expect(content).toContain(`(function (__unused_webpack_module, __unused_webpack_exports, __webpack_require__)`);
	expect(content).toContain(`eval(__webpack_require__.ts(`);
});