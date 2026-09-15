import { pureHelper } from "./pure-lib";
import { impureHelper } from "./impure-lib";

// `keep` is imported by index, so this module is retained and evaluated.
export const keep = "kept";

// Unused + pure -> the call is severed and `pure-lib` is dropped entirely.
export const droppedPure = pureHelper(1);

// Unused + impure -> the call is kept and runs (the side effect must happen).
export const keptImpure = impureHelper(2);
