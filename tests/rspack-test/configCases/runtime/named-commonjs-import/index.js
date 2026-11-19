import { answer } from "./data.cjs";

it("should import a named export without the default interop helper", () => {
	expect(answer).toBe(42);
	const source = require("fs").readFileSync(__filename, "utf-8");
	expect(source).not.toMatch(/^\/\/ (?:webpack|rspack)\/runtime\/compat_get_default_export$/m);
});
