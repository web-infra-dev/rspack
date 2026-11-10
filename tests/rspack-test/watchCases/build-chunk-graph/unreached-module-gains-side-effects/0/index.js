import "./x";
it("runs a newly discovered startup side effect before the async import", async () => {
	if (Number(WATCH_STEP) === 1) expect(globalThis.zRan).toBe(true);
	const { value } = await import("./w");
	expect(value).toBe(1);
});
