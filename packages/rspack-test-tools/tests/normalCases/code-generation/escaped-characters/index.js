import * as X1 from "./module";
import { default as X2 } from "./module";

it("should generate valid code", () => {
	expect(X1["x\\"]).toBe(42);
	expect(X2["x\\"]).toBe(42);
});
