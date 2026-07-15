import "./shadow";

export const marker = 1;

// This module is imported for an export, so these local declarations are kept
// in the emitted module. Side-effect-only imports are covered by the sibling
// modules in this case.

let unusedBoolean = Boolean("hello");
let unusedArrIsArray = Array.isArray([1, 2, 3]);
let unusedObjectKeys = Object.keys({ a: 1 });
let unusedMathSin = Math.sin(1);
let unusedStringChar = String.fromCharCode(65);
let unusedPromiseAccess = Promise;
let unusedNumberProp = Number.foo;
let unusedComputedStatic = Array["isArray"]([]);

function impureArg() {
	console.log("keep");
	return 1;
}

let unusedConstructor = new Set();
let unusedPromiseCtor = new Promise();
let unusedUnsupportedStatic = Math.acosh(1);
let unusedPromiseStatic = Promise.withResolvers();
let unusedNumberParseInt = Number.parseInt("1", 10);
let unusedObjectIs = Object.is(1, 2);
globalThis.isFinite(1);

let dynamic = "isArray";
let unusedDynamicStatic = Array[dynamic]([]);
let unusedBoolImpure = Boolean(impureArg());
