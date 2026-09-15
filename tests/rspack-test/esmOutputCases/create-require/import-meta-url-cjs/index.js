import moduleDefault, { createRequire as r } from "module";
import * as moduleNs from "module";

it("should parse createRequire import.meta.url in CJS output", () => {
	const requireFromAlias = r(import.meta.url);
	expect(requireFromAlias("./a")).toBe("root");

	const requireFromNamespace = moduleNs.createRequire(import.meta.url);
	expect(requireFromNamespace("./a")).toBe("root");

	const requireFromDefault = moduleDefault.createRequire(import.meta.url);
	expect(requireFromDefault("./a")).toBe("root");
});
