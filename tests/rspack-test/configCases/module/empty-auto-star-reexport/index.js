import { dynamic } from "./barrel";
import emptyDefault from "./empty";

it("keeps empty javascript/auto modules as CommonJS", () => {
	const required = require("./empty");

	expect(emptyDefault).toBe(required);
	expect(required).toEqual({});
	expect(required.__esModule).toBeUndefined();
});

it("keeps non-empty dynamic reexports dynamic", () => {
	expect(dynamic).toBe("dynamic");
});
