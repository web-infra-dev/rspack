import { Worker } from "worker_threads";

it("should leave top-level this alone in a module worker", async () => {
	const worker = new Worker(new URL("./w.js", import.meta.url), {
		type: "module"
	});
	try {
		const result = await new Promise((resolve, reject) => {
			worker.once("message", resolve);
			worker.once("error", reject);
		});
		expect(result).toEqual({
			thisIsUndefined: true,
			dependency: { value: 1, thisIsExports: true }
		});
	} finally {
		await worker.terminate();
	}
});
