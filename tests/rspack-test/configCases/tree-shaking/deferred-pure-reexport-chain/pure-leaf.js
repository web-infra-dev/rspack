// A side-effects-free helper reached through the SAME multi-hop re-export chain.
// Its only caller (`unusedPure` in mod.js) is unused, so correct chain
// resolution proves it pure, severs the call, and tree-shakes this whole module
// — including the marker string below — out of the bundle.
//
// This is the discriminating part of the test: if `deferred_pure_check_is_impure`
// stops following the re-export chain, the deferred check cannot resolve and
// falls back to the conservative "keep", so this module (and the marker) would
// survive and the index.js assertion fails. The impure side alone cannot catch
// that regression, because the fallback keeps the impure call either way.
export function pureImpl() {
	return "PURE_REEXPORT_CHAIN_MARKER";
}
