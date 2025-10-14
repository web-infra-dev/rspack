import { greeting } from "./module.js";
import update from "@rspack/test-tools/helper/legacy/update.esm.js";

import.meta.webpackHot.accept(["./module.js"]);

it("should update a simple ES module with HMR", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(greeting).toBe("Hello World!");

	NEXT(update(done, true, () => {
		// After HMR update, we need to re-import the module in ESM
		import("./module.js").then(updatedModule => {
			expect(updatedModule.greeting).toBe("Hello HMR!");
			done();
		}).catch(done);
	}));
}));

it("should have HMR runtime available in ESM output", () => {
	expect(typeof import.meta.webpackHot.accept).toBe("function");
	expect(typeof import.meta.webpackHot.decline).toBe("function");
	expect(typeof import.meta.webpackHot.dispose).toBe("function");
});
