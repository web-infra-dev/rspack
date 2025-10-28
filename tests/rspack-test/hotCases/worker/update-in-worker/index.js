it("should support hot module replacement in WebWorkers", async () => {
	const worker = new Worker(new URL("worker.js", import.meta.url));
	worker.onmessage = async ({ data: msg }) => {
		switch (msg) {
			case "next":
				await NEXT_HMR();
				worker.postMessage("next");
				break;
			case "done":
				await worker.terminate();
				break;
			default:
				throw new Error(`Unexpected message: ${msg}`);
		}
	};
	worker.postMessage("test");
});
