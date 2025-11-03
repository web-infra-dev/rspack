const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createConfigCase(name, src, dist);
});

const formatHeapUsed = (heap) => {
  return `${Math.floor(heap / 1024 / 1024)} MB heap used`;
};

if (process.env.WASM) {
	afterEach(() => {
		console.log(formatHeapUsed(process.memoryUsage().heapUsed));
	});
}

