import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
let extraArgRan = false;
const extraArgRequire = createRequire(import.meta.url, (extraArgRan = true));

export const resolved = require.resolve("path");
export const extraArgResolved = extraArgRequire.resolve("path");

it("keeps a valid created require in CommonJS output when requireResolve is disabled", () => {
	expect(resolved).toBe("path");
	expect(extraArgResolved).toBe("path");
	expect(extraArgRan).toBe(true);
});
