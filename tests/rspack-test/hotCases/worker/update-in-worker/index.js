it("should support hot module replacement in WebWorkers", async () => {
	const worker = new Worker(new URL("worker.js", import.meta.url));
	await new Promise((resolve, reject) => {
		worker.onmessage = async ({ data: msg }) => {
			try {
				switch (msg) {
					case "next":
						await NEXT_HMR();
						worker.postMessage("next");
						break;
					case "done":
						await worker.terminate();
						resolve();
						break;
					default:
						reject(new Error(`Unexpected message: ${msg}`));
				}
			} catch (e) {
				reject(e);
			}
		};
		worker.postMessage("test");
	});
});
