import value from "./file";

it("should wait for imports started before checking for an update", async () => {
	let release;
	const barrier = new Promise((resolve) => {
		release = resolve;
	});
	__webpack_require__.f.blockingTest = (_, promises) => promises.push(barrier);
	const pending = import("./chunk");
	let applied = false;
	module.hot.accept("./file", () => {
		applied = true;
	});
	module.hot.addStatusHandler((status) => {
		if (status === "prepare") {
			expect(applied).toBe(false);
			setTimeout(() => {
				expect(applied).toBe(false);
				delete __webpack_require__.f.blockingTest;
				release();
			}, 20);
		}
	});
	await NEXT_HMR();
	expect(applied).toBe(true);
	expect(value).toBe(2);
	expect((await pending).default).toBe(42);
});
