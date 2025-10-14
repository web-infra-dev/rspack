it("should create a conditional import when accepted", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	if (Math.random() < 0) new Worker(new URL("worker.js", import.meta.url));
	import("./module")
		.then(module =>
			module.test(callback => {
				NEXT(require("@rspack/test-tools/helper/legacy/update")(done, undefined, callback));
			}, done)
		)
		.catch(done);
}));
