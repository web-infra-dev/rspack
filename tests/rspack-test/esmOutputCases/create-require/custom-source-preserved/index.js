import { makeRequire } from "./shim.js";

const require = makeRequire(import.meta.url);

export const customMember = require.a;

it("keeps an unhandled created require from a custom source", () => {
	expect(customMember).toBe("custom");
});
