import "./a.js";
import "./unrelated.js";

it("should not persist stale issuer updates across rebuilds", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
	if (COMPILER_INDEX === 1) await NEXT_START();
	if (COMPILER_INDEX === 2) await NEXT_START();
});
---
import "./b.js";
import "./unrelated.js";

it("should not persist stale issuer updates across rebuilds", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
	if (COMPILER_INDEX === 1) await NEXT_START();
	if (COMPILER_INDEX === 2) await NEXT_START();
});
---
import "./b.js";
import "./unrelated.js";

it("should not persist stale issuer updates across rebuilds", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
	if (COMPILER_INDEX === 1) await NEXT_START();
	if (COMPILER_INDEX === 2) await NEXT_START();
});
---
import "./b.js";
import "./unrelated.js";

it("should not persist stale issuer updates across rebuilds", async () => {
	if (COMPILER_INDEX === 0) await NEXT_START();
	if (COMPILER_INDEX === 1) await NEXT_START();
	if (COMPILER_INDEX === 2) await NEXT_START();
});
