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

	// This keeps the dependencies of the custom runtime from being included
	expect(__webpack_modules__["./error.js"]).toBeFalsy();
});

it("should transform typeof require indent", () => {
	const { testTypeofRequire } = require("./typeof-require");
	expect(testTypeofRequire()).toBe(true);
});

it("should rewrite require url successfully", () => {
	const a = {
		v1: () => {
			return require("./typeof-require").testTypeofRequire();
		},
		v2: require("./typeof-require").testTypeofRequire()
	};
	function b() {
		const a = {
			v1: require("./typeof-require").testTypeofRequire()
		};
		expect(a.v1).toBe(true);
	}
	expect(a.v1()).toBe(true);
	expect(a.v2).toBe(true);
	b();
});

import context from "./context";

it("should rewrite require url successfully in context", () => {
	expect(context.foo).toBe("foo");
});
