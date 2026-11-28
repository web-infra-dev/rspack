import { value } from "./a.js";

export const sync = value;
export const async = () => import("./a-async.js").then(m => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("a");
	expect(await async()).toBe("a");
	if (COMPILER_INDEX === 0) await rebuildInSameCompiler();
});
---
import { value } from "./b.js";

export const sync = value;
export const async = () => import("./b-async.js").then(m => m.default);
---
import { value } from "./b.js";

export const sync = value;
export const async = () => import("./b-async.js").then(m => m.default);
---
import { value } from "./a.js";

export const sync = value;
export const async = () => import("./a-async.js").then(m => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("a");
	expect(await async()).toBe("a");
	if (COMPILER_INDEX === 0) await rebuildInSameCompiler();
});
