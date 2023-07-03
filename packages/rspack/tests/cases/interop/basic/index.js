import a_default from "./a";
import * as a_all from "./a";
import * as esm from "./esm";

it("should exports __esModule", function () {
	expect(esm.__esModule).toBe(true);
});

it("should have interop when import cjs", function () {
	expect(a_default.test).toBe(a_all.test);
});
