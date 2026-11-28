import { Worker as NodeWorker } from "worker_threads";
import { Worker as PrefixedWorker } from "node:worker_threads";

it("should preserve CommonJS this in Node worker entries", async () => {
	const workers = [
		new NodeWorker(new URL("./worker.cjs", import.meta.url)),
		new PrefixedWorker(new URL("./worker.cjs", import.meta.url))
	];
	try {
		const results = await Promise.all(workers.map(worker => new Promise((resolve, reject) => {
			worker.once("message", resolve);
			worker.once("error", reject);
		})));
		for (const result of results) {
			expect(result).toEqual({
				thisIsExports: true,
				value: 42,
				globalUntouched: true,
				dependency: { value: 1, thisIsExports: true }
			});
		}
	} finally {
		await Promise.all(workers.map(worker => worker.terminate()));
	}
});
