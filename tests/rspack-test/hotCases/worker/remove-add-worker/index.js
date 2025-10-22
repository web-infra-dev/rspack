import compute from "./compute";

it("should support adding and removing runtimes", async () => {
	expect(await compute()).toBe(42);
	await NEXT_HMR();
	expect(await compute()).toBe(42);
	await NEXT_HMR();
	expect(await compute()).toBe(42);
	await NEXT_HMR();
	expect(await compute()).toBe(42);
	await NEXT_HMR();
	expect(await compute()).toBe(42);
	await NEXT_HMR();
	expect(await compute()).toBe(42);
});

import.meta.webpackHot.accept("./compute");
