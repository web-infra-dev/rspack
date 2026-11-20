export * from "./value";

Object();
Object(1);
Object?.(1);
Boolean(0);
String(1);
Number("1");
Symbol("a");
isNaN(1);
isFinite(1);
parseInt("1", 10);
parseFloat("1.5");
encodeURI("a b");
encodeURIComponent("a b");
decodeURI("a%20b");
decodeURIComponent("a%20b");

Array.isArray([]);
Array.of(1);
Number.isFinite(1);
Number.isInteger(1);
Number.isNaN(1);
Number.isSafeInteger(1);
Number.parseFloat("1.5");
Number.parseInt("1", 10);
Object.getOwnPropertyDescriptor({}, "a");
Object.getOwnPropertyDescriptors({});
Object.getOwnPropertyNames({});
Object.getOwnPropertySymbols({});
Object.getPrototypeOf({});
Object.hasOwn({}, "a");
Object.is(1, 1);
Object.isExtensible({});
Object.isFrozen({});
Object.isSealed({});
Object.keys({});
String.fromCharCode(97);
String.fromCodePoint(97);
Symbol.for("a");
Symbol.keyFor(Symbol.for("a"));
Object.entries({ a: 1 });
Object.values({ a: 1 });
Object.values({
	get a() {
		return 1;
	}
});
Math.max(1, 2);
Math.random();
Math.E;
Math.unknownProperty;
Date();

new Object();
new Boolean(1);
new String("a");
new Number(1);
new Date(0);
new RegExp("a", "g");
new Array(1);
new ArrayBuffer(8);
new DataView(new ArrayBuffer(8));
new Error("a");
new EvalError("a");
new RangeError("a");
new ReferenceError("a");
new SyntaxError("a");
new TypeError("a");
new URIError("a");
new URLSearchParams();
new URLSearchParams("a=1");
new Set();
new Set([1]);
new WeakSet();
new Map();
new Map([[1, 2]]);
new WeakMap();
new Uint8Array();
new Uint8Array(2);
new Float64Array([1, 2]);
new Error("a", {
	get cause() {
		return 1;
	}
});

const local = Object({});

export const tau = Math.PI * 2;
export const boxed = Object(1);
export const nested = Object(Boolean(String(1)));
export const keys = Object.keys({ a: 1 });
export const set = new Set([1]);
export default Object(2);
