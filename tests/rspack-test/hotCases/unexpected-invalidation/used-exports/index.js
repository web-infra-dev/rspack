import { ghi } from "./subject";
import value from "./module";

it("should not invalidate subject in unrelated locations", async () => {
	expect(ghi).toBe(42);
	expect(value).toBe(40);
	await NEXT_HMR();
	expect(ghi).toBe(42);
	expect(value).toBe(41);
});

import.meta.webpackHot.accept("./module");