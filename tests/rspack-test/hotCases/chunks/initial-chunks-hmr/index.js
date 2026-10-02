import "lib-js/a";
import.meta.webpackHot.accept();

it("should work if there are new initial chunks", async () => {
	expect((await import("./initial")).value).toBe("a");
	await NEXT_HMR();
});
---
import "lib-js/a";

it("should work if there are new initial chunks", async () => {
	// HMR must install the new initial dependency before import() can ensure it.
	expect(__webpack_modules__["./node_modules/lib-js/b.js"]).toBeTypeOf("function");
	expect((await import("./initial")).value).toBe("b");
});
