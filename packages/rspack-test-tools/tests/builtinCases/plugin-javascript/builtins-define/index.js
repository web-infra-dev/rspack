// import assert from "assert";
// import { equal } from "assert";
import DO_NOT_CONVERTED8, { DO_NOT_CONVERTED7 } from "./lib";

const lib = require("./lib");
const { DO_NOT_CONVERTED9 } = require("./lib");

equal(TRUE, true);
// require("assert").deepStrictEqual(FALSE, false);
assert.deepStrictEqual(NUMBER_ADD, 5);
assert.deepStrictEqual(NULL, null);
assert.deepStrictEqual(UNDEFINED, undefined);
// assert.equal(FUNCTION(5), 6);
// assert.equal(typeof FUNCTION, "function");
assert.deepStrictEqual(NUMBER, 100.05);
assert.deepStrictEqual(ZERO, 0);
let ZERO_OBJ = { ZERO: 0 };
assert.deepStrictEqual(ZERO_OBJ.ZERO, 0);
assert.deepStrictEqual(ZERO_OBJ[ZERO], undefined);
assert.deepStrictEqual(ZERO_OBJ[0], undefined);
assert.deepStrictEqual(ZERO_OBJ["ZERO"], 0);
assert.deepStrictEqual(BIGINT, 10000n);
assert.deepStrictEqual(BIGINT2, 100000000000n);
assert.deepStrictEqual(POSITIVE_ZERO, 0);
assert.deepStrictEqual(NEGATIVE_ZERO, -0);
assert.deepStrictEqual(POSITIVE_NUMBER, 100.25);
assert.deepStrictEqual(NEGATIVE_NUMBER, -100.25);
assert.deepStrictEqual(STRING, "string");
assert.deepStrictEqual(EMPTY_STRING, "");
assert.deepStrictEqual(REGEXP, /abc/i);
assert.deepStrictEqual(ZERO.ABC, undefined);

let error_count = 0;
try {
	error_count += 1;
	MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 1);

try {
	error_count += 1;
	MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 2);

assert.deepStrictEqual(ARRAY, [300, ["six"]]);
assert.deepStrictEqual(ARRAY[0], 300);
assert.deepStrictEqual(ARRAY[0][1], undefined);
assert.deepStrictEqual(ARRAY[1], ["six"]);
assert.deepStrictEqual(ARRAY[1][0], "six");
assert.deepStrictEqual(ARRAY[1][0][0], "s");
assert.deepStrictEqual(ARRAY[ONE], ["six"]);
assert.deepStrictEqual(ARRAY[ARRAY], undefined);

assert.deepStrictEqual(OBJECT, {
	UNDEFINED: undefined,
	REGEXP: /def/i,
	STR: "string",
	OBJ: { NUM: 1 }
});
assert.deepStrictEqual(OBJECT.OBJ, { NUM: 1 });
assert.deepStrictEqual(OBJECT.OBJ.NUM, 1);
assert.deepStrictEqual(OBJECT.UNDEFINED, undefined);
assert.deepStrictEqual(OBJECT.REGEXP, /def/i);
assert.deepStrictEqual(OBJECT.STR, "string");
assert.deepStrictEqual(OBJECT.AAA, undefined);

assert.deepStrictEqual(P1.P2.P3, 301);
assert.deepStrictEqual(P1.P2.P4, "302");
assert.deepStrictEqual(P1, 303);
assert.deepStrictEqual(P1.P2, 304);

assert.deepStrictEqual(P1.P4, undefined); // "303.P4"

try {
	error_count += 1;
	P4.P1;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 3);

assert.deepStrictEqual(P1.P2.P4.P1, undefined);
assert.deepStrictEqual(P1.P2.P4.P3, undefined);
assert.deepStrictEqual(P1.P2.P4.P4, undefined);

const DO_NOT_CONVERTED = 201;
assert.deepStrictEqual(DO_NOT_CONVERTED, 201);
let { DO_NOT_CONVERTED2 } = { DO_NOT_CONVERTED2: 202 };
assert.deepStrictEqual(DO_NOT_CONVERTED2, 202);
const { c: DO_NOT_CONVERTED3 } = { c: 203 };
assert.deepStrictEqual(DO_NOT_CONVERTED3, 203);
try {
	error_count += 1;
	DO_NOT_CONVERTED4;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 4);
let DO_NOT_CONVERTED4 = 204;

const USELESS = {
	ZERO: 0
};
{
	const A = DO_NOT_CONVERTED4;
	assert.deepStrictEqual(A, 204);

	const DO_NOT_CONVERTED3 = 205;
	assert.deepStrictEqual(DO_NOT_CONVERTED3, 205);

	const B = ZERO;
	assert.deepStrictEqual(B, 0);

	let IN_BLOCK = 2;
	assert.deepStrictEqual(IN_BLOCK, 2);

	{
		{
			{
				assert.deepStrictEqual(SHOULD_CONVERTED, 205);
			}
		}
	}
}

try {
	error_count += 1;
	IN_BLOCK;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 5);

assert.deepStrictEqual(USELESS, { ZERO: 0 });
assert.deepStrictEqual({}.DO_NOT_CONVERTED5, undefined);
assert.deepStrictEqual(M1.M2.M3.DO_NOT_CONVERTED6, undefined);
assert.deepStrictEqual(DO_NOT_CONVERTED7, 402);
assert.deepStrictEqual(DO_NOT_CONVERTED8, 401);
assert.deepStrictEqual(DO_NOT_CONVERTED9, 403);
assert.deepStrictEqual(lib.DO_NOT_CONVERTED9, 403);

try {
	error_count += 1;
	M1;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 6);

// try {
//   error_count += 1;
//   SHOULD_CONVERTED = 205; // syntax error
//   error_count += 1;
// } catch (err) {
// }
// deepStrictEqual(error_count, 6);

// try {
//   error_count += 1;
//   SHOULD_CONVERTED = SHOULD_CONVERTED = 205; // syntax error
//   error_count += 1;
// } catch (err) {
// }
// deepStrictEqual(error_count, 7);
try {
	error_count += 1;
	aa = SHOULD_CONVERTED;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 7);

assert.deepStrictEqual(SHOULD_CONVERTED == 205, true);
assert.deepStrictEqual(207 == SHOULD_CONVERTED, false);

try {
	error_count += 1;
	CONVERTED_TO_MEMBER;
	error_count += 1;
} catch (err) {}
assert.deepStrictEqual(error_count, 8);

// just make sure `MemberExpr` fold success.
console.log(console.log(console.log));

// TODO: recursive
// assert.equal(wurst, unde);
// assert.equal(suppe, wurst);
