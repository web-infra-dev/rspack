// Shadowing: `Set`/`Map` below are LOCAL bindings, not the real globals.
// The unresolved-context check must keep these calls despite matching names.
function impureSet() { console.log("keep"); return class {} }
const Set = impureSet();
let shadowedSet = new Set();

const Map = impureSet();
let shadowedMap = new Map();
