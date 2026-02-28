import m from "./module";

it("should not dispose shared modules when a chunk is removed", async () => {
	const chunk1 = await import("./chunk1");
	await NEXT_HMR();
	expect(m).toBe(42);
	expect(chunk1).toMatchObject({
		active: true
	});
});

import.meta.webpackHot.accept("./module");