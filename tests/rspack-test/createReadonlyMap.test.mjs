import { createReadonlyMap } from "../../packages/rspack/src/util/createReadonlyMap.ts";

describe("createReadonlyMap", () => {
	it("should support has() and size on the returned ReadonlyMap", () => {
		const backing = new Map([
			["main", { id: 1 }],
			["empty", undefined],
		]);
		const map = createReadonlyMap({
			keys: () => backing.keys(),
			get: key => backing.get(key),
		});

		expect(map.size).toBe(2);
		expect(map.has("main")).toBe(true);
		expect(map.has("empty")).toBe(true);
		expect(map.has("missing")).toBe(false);
		expect(Array.from(map.entries())).toEqual([
			["main", { id: 1 }],
			["empty", undefined],
		]);
	});
});
