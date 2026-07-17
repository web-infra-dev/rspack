import { createRequire } from "node:module";

const req = createRequire(import.meta.url);

it("should delegate import.meta.url to DefinePlugin when createRequire is preserved", () => {
	expect(req.resolve("path")).toBe("path");
});
