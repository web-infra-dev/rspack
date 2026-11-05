import "./a.js";
import "./b.js";
import "./unrelated.js";

it("should not persist stale issuer updates across rebuilds", async () => {
	if (COMPILER_INDEX === 0) {
		expect(globalThis.unrelated).toBe("initial");
		await rebuildIssuers();
	} else {
		expect(globalThis.unrelated).toBe("second edit");
	}
});
