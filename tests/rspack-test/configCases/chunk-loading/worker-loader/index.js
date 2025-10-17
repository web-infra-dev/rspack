import fs from "fs"
import Worker from "worker-rspack-loader!./worker.js"

it("should contain import-scripts chunkLoading runtime", () => {
	Worker;
	let file = fs.readFileSync(__dirname + "/bundle0.worker.js", "utf-8")
	expect(file).toContain("__webpack_require__.f.i")
})
