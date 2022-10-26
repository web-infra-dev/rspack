import a_default from "./a";
import * as a_all from "./a";

it("should exports __esModule", function () {
	expect(exports.__esModule).toBe(true);
});

it("should have interop when import cjs", function () {
	expect(a_default.test).toBe(a_all.test);
});
