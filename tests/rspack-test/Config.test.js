const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createConfigCase(name, src, dist);
});

const formatHeapUsed = (heap) => {
  return `${Math.floor(heap / 1024 / 1024)} MB heap used`;
};

if (process.env.WASM) {
	afterAll(() => {
		console.log(formatHeapUsed(process.memoryUsage().heapUsed));
	});
}
process.on('exit', (code) => {
	console.trace('exit', process.pid, code)
});

