export const marker = 1;

// These should all be tree-shaken as unused pure expressions.
let unusedSet = new Set();
let unusedMap = new Map();
let unusedWeakMap = new WeakMap();
let unusedWeakSet = new WeakSet();
let unusedTyped = new Uint8Array(16);
let unusedObjKeys = Object.keys({ x: 1 });
let unusedArrIsArray = Array.isArray([1, 2, 3]);
let unusedString = String(123);

// Shadowed — must NOT be treated as pure.
function sideEffect() { console.log("keep"); return class {} }
const ShadowedSet = sideEffect();
let shadowed = new ShadowedSet();

// Impure argument — must NOT be tree-shaken.
function impureArg() { console.log("keep"); return 1; }
let unusedWithImpureArg = new Set([impureArg()]);
