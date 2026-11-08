// `identity` is intentionally NOT auto-detectable as side-effects-free — the `if`
// statement makes the side-effects-free auto-analysis bail. So it is treated as
// pure ONLY via the consumer-side parser.pureFunctions hint (the
// trust-at-call-site Direct path), never via deferred resolution of its body.
//
// That is what makes this test discriminating: if the Direct path regresses to
// normal deferred resolution, `identity` is classified impure here, its unused
// call is kept, and the sentinel below survives in the bundle — which the drop
// assertion in index.js then catches.
export function identity(x) {
	if (x === "pure-functions-consumer-hint::wrapper-sentinel") return undefined;
	return x;
}
