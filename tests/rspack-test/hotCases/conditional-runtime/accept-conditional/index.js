it("should create a conditional import when accepted", async () => {
	if (Math.random() < 0) new Worker(new URL("worker.js", import.meta.url));
	const m = await import("./module");
	await m.test(NEXT_HMR);
});

module.hot.accept("./module");