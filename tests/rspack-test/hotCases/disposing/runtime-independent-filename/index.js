import m from "./module";

it("should not dispose shared modules when a chunk from a different runtime is removed", async () => {
	const chunk1 = await import("./chunk1");
	await NEXT_HMR();
	expect(m).toBe(42);
	expect(chunk1).toMatchObject({
		active: false
	});
});

import.meta.webpackHot.accept("./module");
