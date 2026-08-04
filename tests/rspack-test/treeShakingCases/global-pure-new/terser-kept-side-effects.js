// Terser `compress.unsafe` does not treat `new` constructors, unsupported
// static functions, `globalThis.*`, or dynamic member calls as pure.
new Object();
new Set();
new Promise();
new Number(1);
Array.of(1, 2);
ArrayBuffer.isView(new Uint8Array());
Date.now();
Math.acosh(1);
Number.parseInt("1", 10);
Object.is(1, 2);
Promise.withResolvers();
String.fromCodePoint(65);
globalThis.isFinite(1);
globalThis.Math.cos(1);
Promise.foo;

const dynamic = "isArray";
Array[dynamic]([]);
