it("should preserve required exports assigned to top-level this under usedExports", async () => {
	const worker = new Worker(new URL("./worker.js", import.meta.url));
	worker.postMessage("ask");
	const result = await new Promise(resolve => {
		worker.onmessage = event => resolve(event.data);
	});
	expect(result).toBe(42);
	await worker.terminate();
});
