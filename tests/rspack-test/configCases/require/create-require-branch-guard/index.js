import * as module from "./shim.js";
import { createRequire } from "./shim.js";

if ("createRequire" in module) {
	const req = createRequire(import.meta.url);
	globalThis.__createRequireUnknownMember = req.unknown;
}

it("should keep the createRequire import branch guard", () => {
	expect(globalThis.__createRequireUnknownMember).toBeUndefined();
});
