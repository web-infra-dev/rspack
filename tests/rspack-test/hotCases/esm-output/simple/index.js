import { greeting } from "./module.js";

import.meta.webpackHot.accept(["./module.js"]);

it("should update a simple ES module with HMR", async () => {
	expect(greeting).toBe("Hello World!");
	await NEXT_HMR();
	const updatedModule = await import("./module.js");
	expect(updatedModule.greeting).toBe("Hello HMR!");
});

it("should have HMR runtime available in ESM output", () => {
	expect(typeof import.meta.webpackHot.accept).toBe("function");
	expect(typeof import.meta.webpackHot.decline).toBe("function");
	expect(typeof import.meta.webpackHot.dispose).toBe("function");
});
