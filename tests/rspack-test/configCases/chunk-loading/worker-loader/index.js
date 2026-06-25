import fs from "fs"
import Worker from "worker-rspack-loader!./worker.js"

it("should contain import-scripts chunkLoading runtime", () => {
	Worker;
	let file = fs.readFileSync(__dirname + "/bundle0.worker.js", "utf-8")
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(file).toContain("ensureChunkHandlers.i")
	} else {
		expect(file).toContain("__webpack_require__.f.i")
	}
})
