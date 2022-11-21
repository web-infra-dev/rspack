import { foo } from "./foo";
const cjs = require("./cjs");

it("should not replace custom require with __webpack_require__", () => {
	expect(foo).toBe("foo");
	expect(cjs).toBe("cjs");

	const ERROR_ID = "./error";

	function require(id) {
		if (ERROR_ID === id) {
			return "success";
		}

		throw new Error("Failed to require " + id);
	}

	expect(require(ERROR_ID)).toBe("success");
});
