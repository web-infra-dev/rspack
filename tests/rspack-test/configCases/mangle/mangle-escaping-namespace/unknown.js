export * from "./unknown-cjs";
export const known = "known";
export const canMangle = __webpack_exports_info__.known.canMangle;

// Exercise an unknown namespace outside the concatenation group.
eval("");
