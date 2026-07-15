// Shadowing: these are LOCAL bindings, not the real globals.
// The unresolved-context check must keep these calls despite matching names.
function impureValue() {
	console.log("keep");
	return () => 1;
}
const Boolean = impureValue();
let shadowedBoolean = Boolean(1);

const Array = { isArray: impureValue() };
let shadowedArray = Array.isArray([]);
