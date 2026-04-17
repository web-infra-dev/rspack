import "./shadow";

export const marker = 1;

// === Should be tree-shaken (truly pure) ===

// Collections with no args.
let unusedSet = new Set();
let unusedMap = new Map();
let unusedWeakMap = new WeakMap();
let unusedWeakSet = new WeakSet();

// Collections with explicit nullish are also fine (return empty collection).
let unusedNullSet = new Set(null);
let unusedUndefMap = new Map(undefined);

// TypedArrays with non-negative integer literal length.
let unusedTyped = new Uint8Array(16);
let unusedBuf = new ArrayBuffer(0);

// Pure type/identity checks (AnyPureArgs — args themselves must be pure too).
let unusedArrIsArray = Array.isArray([1, 2, 3]);
let unusedObjectIs = Object.is(1, 2);

// String/Object with literal args — no coercion since literals don't have
// custom @@toPrimitive.
let unusedString = String("hello");
let unusedObject = Object("y");

// Boolean is pure regardless of arg shape (ToBoolean never throws).
let unusedBool = Boolean({});
let unusedBoolVar = Boolean(marker);

// Symbol() with primitive description.
let unusedSymbol = Symbol("desc");

// === MUST be kept (throw or have side effects at runtime) ===

// `new Set(1)` throws TypeError (1 is not iterable).
let throwsTypeErrorSet = new Set(1);

// `new Map("foo")` actually throws (string iterates to chars, not [k,v] pairs).
// Even though it's a literal, the gate (NullishOrNoArgs) correctly rejects.
let throwsMap = new Map("foo");

// `new Array(-1)` throws RangeError.
let throwsRangeErrorArr = new Array(-1);

// `new Array(1.5)` throws RangeError.
let throwsArrFractional = new Array(1.5);

// `new Uint8Array(-1)` throws RangeError.
let throwsTypedNeg = new Uint8Array(-1);

// `new Uint8Array(1.5)` throws RangeError.
let throwsTypedFractional = new Uint8Array(1.5);

// `new Date(1n)` throws TypeError (BigInt → Number coercion fails).
let throwsDateBigInt = new Date(1n);

// `new Number(1n)` throws TypeError.
let throwsNumberBigInt = new Number(1n);

// Impure argument — `Boolean(sideEffect())` is pure, but the argument call
// has side effects, so the whole expression must stay.
function impureArg() { console.log("keep"); return 1; }
let unusedWithImpureArg = new Set([impureArg()]);
let unusedBoolImpure = Boolean(impureArg());

// Non-literal arg for typed array — could trigger valueOf coercion.
let dynamic = { length: 16 };
let unusedWithDynamic = new Uint8Array(dynamic);
