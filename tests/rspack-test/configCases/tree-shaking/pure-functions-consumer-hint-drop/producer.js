import { identity } from "./wrapper";
import { payload } from "./payload";

// `wrapped` is never imported. `identity` is pure only via the consumer-side
// parser.pureFunctions hint, and `payload` is side-effects-free, so the whole
// `identity(payload())` call is severed and both `wrapper` and `payload` are
// tree-shaken out of the bundle.
export const wrapped = identity(payload());
