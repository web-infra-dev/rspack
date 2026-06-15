import { createRequire } from "module";
import module from "module";
import * as moduleNs from "module";

it("should parse createRequire import.meta.url in ESM output", () => {
	const require = createRequire(import.meta.url);
	expect(require("./a")).toBe("root");
	expect(createRequire(import.meta.url)("./a")).toBe("root");
	expect(moduleNs.createRequire(import.meta.url)("./a")).toBe("root");
	expect(module.createRequire(import.meta.url)("./a")).toBe("root");
});
