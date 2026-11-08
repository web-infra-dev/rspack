import { impl, pureImpl } from "./hop2";

// `keep` is imported by index, so this module is retained and evaluated.
export const keep = "kept";

// `unusedImpure` is never imported. Its initializer is a deferred pure call to
// `impl` (resolved through the chain to the impure leaf), so it must be kept and
// run — severing it would drop the side effect or leave an undefined("z")
// partial-sever.
export const unusedImpure = impl("z");

// `unusedPure` is also never imported, and `pureImpl` resolves through the same
// chain to the side-effects-free leaf, so its call must be severed and
// `pure-leaf` tree-shaken away.
export const unusedPure = pureImpl();
