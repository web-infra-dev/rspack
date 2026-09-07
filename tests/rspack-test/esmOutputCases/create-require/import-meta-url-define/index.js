import { createRequire } from "node:module";
import { bundledValue, unknownValue } from "./mixed.js";

const req = createRequire(import.meta.url);

it("should align parsed require context with the preserved DefinePlugin argument", () => {
	expect(req.resolve("path")).toBe("path");
	expect(bundledValue).toBe("defined context");
	expect(unknownValue).toBeUndefined();
});
