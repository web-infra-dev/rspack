import { keep } from "./mod";

const fs = require("fs");

it("resolves impure (kept) and pure (dropped) callees through a re-export chain", () => {
	expect(keep).toBe("kept");

	// Impure side: the retained `unusedImpure = impl("z")` initializer must have
	// executed exactly once, proving the impure callee was kept through the chain.
	expect(globalThis.__deferredPureReexportChainCalls).toBe(1);

	// Pure side (discriminating): `pureImpl` resolves through the same chain to a
	// side-effects-free leaf, so its unused call is severed and `pure-leaf` is
	// tree-shaken out — the marker must be absent. If chain resolution regresses,
	// the deferred check falls back to "keep" and the marker would survive here.
	const marker = ["PURE", "REEXPORT", "CHAIN", "MARKER"].join("_");
	const source = fs.readFileSync(__filename, "utf-8");
	expect(source).not.toContain(marker);
});
