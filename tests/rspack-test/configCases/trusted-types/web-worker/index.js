const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");


function createWorker() {
	new Worker(new URL("./worker.js", import.meta.url), {
		type: "module"
	});
}

createWorker;

it("should generate correct new Worker statement", async () => {
	const content = fs.readFileSync(__filename, "utf-8");
	const method = "__webpack_require__.tu";
	expect(content).toContain(`new Worker(${method}(new URL(`)
});


function createWorkerWithChunkName() {
	new Worker(/* webpackChunkName: "someChunkName" */new URL("./worker.js", import.meta.url));
}

createWorkerWithChunkName

it("should generate correct new Worker statement with magic comments", async () => {
	const content = fs.readFileSync(__filename, "utf-8");
	const chunkName = "someChunkName";
	expect(content).toContain(`new Worker(/* webpackChunkName: "${chunkName}" */__webpack_require__.tu(new URL(`)
	expect(fs.existsSync(path.join(__dirname, `${chunkName}.js`))).toBeTruthy();
});


function createWorkerWithChunkNameInnner() {
	new Worker(new URL(/* webpackChunkName: "someChunkName2" */ "./worker.js", import.meta.url));
}

createWorkerWithChunkNameInnner

it("should generate correct new Worker statement with magic comments", async () => {
	const content = fs.readFileSync(__filename, "utf-8");
	const chunkName = "someChunkName2";
	expect(content).toContain(`new Worker(__webpack_require__.tu(new URL(/* webpackChunkName: "${chunkName}" */`)
	expect(fs.existsSync(path.join(__dirname, `${chunkName}.js`))).toBeTruthy();
});
