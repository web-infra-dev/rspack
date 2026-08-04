// This module is also imported only for side effects, but these built-ins must
// keep impure arguments alive.
function impureArg() {
	console.log("keep side effects");
	return 1;
}

Boolean(impureArg());
Array.isArray([impureArg()]);
Object.keys({ value: impureArg() });
Math.sin(impureArg());
String.fromCharCode(impureArg());
