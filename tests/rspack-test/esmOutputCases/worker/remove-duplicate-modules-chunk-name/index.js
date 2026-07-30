import { Worker } from "node:worker_threads";

it("should preserve a duplicated worker entry name", async () => {
	const worker = new Worker(
		new URL(
			/* webpackChunkName: "named-worker" */ "./lib.js",
			import.meta.url
		)
	);
	let result;

	try {
		result = await new Promise((resolve, reject) => {
			worker.on("message", resolve);
			worker.on("error", reject);
			worker.on("exit", code => {
				reject(new Error(`Worker stopped before responding with exit code ${code}`));
			});
		});
	} finally {
		await worker.terminate();
	}

	expect(result).toBe("worker result");
});
