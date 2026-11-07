import { moduleIds, values } from "./lib.js";

it("keeps nested runtime tables separate when concatenating modules", async () => {
	expect(moduleIds).toEqual([["prebuilt"], ["prebuilt"]]);
	expect(values).toEqual([42, 7]);
	const lazy = await import("./lazy.js");
	expect(lazy.default).toBe(7);
});
