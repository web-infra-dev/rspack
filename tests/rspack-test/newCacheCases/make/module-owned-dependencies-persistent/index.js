import { value } from "./a.js";

export const sync = value;
export const async = () => import("./a-async.js").then((m) => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("a");
	expect(await async()).toBe("a");
	if (COMPILER_INDEX === 1) await NEXT_START();
});
---
import { value } from "./b.js";

export const sync = value;
export const async = () => import("./b-async.js").then((m) => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("b");
	expect(await async()).toBe(COMPILER_INDEX === 3 ? "updated" : "b");
	if (COMPILER_INDEX === 2 || COMPILER_INDEX === 3) await NEXT_START();
});
---
import { value } from "./b.js";

export const sync = value;
export const async = () => import("./b-async.js").then((m) => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("b");
	expect(await async()).toBe(COMPILER_INDEX === 3 ? "updated" : "b");
	if (COMPILER_INDEX === 2 || COMPILER_INDEX === 3) await NEXT_START();
});
---
import { value } from "./a.js";

export const sync = value;
export const async = () => import("./a-async.js").then((m) => m.default);

it("should retain dependency and block objects across graph replacement", async () => {
	expect(sync).toBe("a");
	expect(await async()).toBe("a");
	if (COMPILER_INDEX === 1) await NEXT_START();
});
