// Second hop: deferred_pure_check_is_impure must follow this whole chain
// (hop2 -> hop1 -> impure-leaf / pure-leaf) to resolve each callee's real target.
export { impl, pureImpl } from "./hop1";
