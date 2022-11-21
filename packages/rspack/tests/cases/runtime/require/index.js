import { foo } from "./foo";

it("should compile monaco successfully (issue #1216)", () => {
	const { insane } = require("./monaco");
	expect(!!insane).toBeTruthy();
});

it("modules required by custom module runtimes should not be included", () => {
	const cjs = require("./cjs");
	expect(foo).toBe("foo");
	expect(cjs).toBe("cjs");

	function wrapper(module, require, exports) {
		expect(require("./error")).toBe("success");
	}

	wrapper(
		{},
		function require(id) {
			if (id === "./error") {
				return "success";
			}

			throw new Error("Failed to require " + id);
		},
		{}
	);

	expect(eval("__webpack_require__.m")["./error.js"]).toBeFalsy();
});
