// This module is imported only for side effects. With builtinPureGlobals enabled,
// all top-level statements below follow Terser's unsafe native-object tables
// and the whole module should be dropped.
Boolean("hello");
Array.isArray([1, 2, 3]);
Array["isArray"]([]);
Object.keys({ a: 1 });
Math.sin(1);
String.fromCharCode(65);
Number.foo;
Math.foo;
JSON.foo;
eval.foo;
Promise;
