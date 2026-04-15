import "./shadow";

export const marker = 1;

// All side-effect-free — should be tree-shaken.
let unusedSet = new Set();
let unusedMap = new Map();
let unusedWeakMap = new WeakMap();
let unusedWeakSet = new WeakSet();
let unusedTyped = new Uint8Array(16);
let unusedArrIsArray = Array.isArray([1, 2, 3]);
let unusedString = String("hello");
let unusedBool = Boolean({});
let unusedSymbol = Symbol("desc");

// Impure argument — Set iterates the array, but the array contains a call
// expression with side effects. Must be kept.
function impureArg() { console.log("keep"); return 1; }
let unusedWithImpureArg = new Set([impureArg()]);

// Non-literal arg — could trigger valueOf/toString / iterator. Must be kept.
let dynamic = { length: 16 };
let unusedWithDynamic = new Uint8Array(dynamic);
