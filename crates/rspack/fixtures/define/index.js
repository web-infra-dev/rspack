import assert from "assert";
import DO_NOT_CONVERTED8, { DO_NOT_CONVERTED7 } from "./lib";
const { DO_NOT_CONVERTED9 } = require("./lib");
const lib = require("./lib");

TRUE;
FALSE;
NUMBER_ADD;
NULL;
UNDEFINED; // tags
// assert.equal(FUNCTION(5), 6);
// assert.equal(typeof FUNCTION, "function");
NUMBER;
ZERO; // tags

({ ZERO: 0 }.ZERO);
({ ZERO: 0 }.ZERO);
({ ZERO: 0 }[ZERO]); // undefined
({ ZERO: 0 }[0]); // undefined
({ ZERO: 0 }["ZERO"]); // 0
BIGINT;
BIGINT2;
POSITIVE_ZERO;
NEGATIVE_ZERO;
POSITIVE_NUMBER;
NEGATIVE_NUMBER;
EMPTY_STRING;
REGEXP; // tags

ZERO.ABC; //should converted to `0.REGEXP`
MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.ZERO;
MEMBER_PROPS_SHOULD_DO_NOT_CONVERTED.REGEXP.REGEXP;

ARRAY;
ARRAY[0];
ARRAY[0][1];
ARRAY[1];
ARRAY[1][0];
ARRAY[1][0][0];
ARRAY[ONE];
ARRAY[ARRAY];

OBJECT; // tags
OBJECT.OBJ;
OBJECT.OBJ.NUM;
OBJECT.UNDEFINED;
OBJECT.REGEXP;
OBJECT.STR;
OBJECT.AAA.BBB;

assert.equal(P1.P2.P3, 301);
assert.equal(P1.P2.P4, "302");
assert.equal(P1, 303);
assert.equal(P1.P2, 304);
P1.P4; // "303.P4"
P4.P1; // do not change
P1.P2.P4.P1.P2; // "302".P1.P2
P1.P2.P4.P3; // "302".P3
P1.P2.P4.P4; // "302".P4

const DO_NOT_CONVERTED = 201;
assert.equal(DO_NOT_CONVERTED, 201);
let { DO_NOT_CONVERTED2 } = { DO_NOT_CONVERTED2: 202 };
assert.equal(DO_NOT_CONVERTED2, 202);
const { c: DO_NOT_CONVERTED3 } = { c: 203 };
assert.equal(DO_NOT_CONVERTED3, 203);
assert.equal(DO_NOT_CONVERTED4, 204);
let DO_NOT_CONVERTED4 = 204;
let USELESS = {
  ZERO: 0,
};
{
  const A = DO_NOT_CONVERTED4;
  assert.equal(A, 204);

  const DO_NOT_CONVERTED3 = 205;
  assert.equal(DO_NOT_CONVERTED3, 205);

  const B = ZERO;
  assert.equal(B, 0);
}

assert.deepStrictEqual(USELESS, { ZERO: 0 });
assert.equal({}.DO_NOT_CONVERTED5, undefined);
assert.equal(M1.M2.M3.DO_NOT_CONVERTED6, undefined);
assert.equal(M1, undefined);
assert.equal(DO_NOT_CONVERTED7, 402);
assert.equal(DO_NOT_CONVERTED8, 401);
assert.equal(DO_NOT_CONVERTED9, 403);
assert.equal(lib.DO_NOT_CONVERTED9, 403);

SHOULD_CONVERTED = 205;
SHOULD_CONVERTED = SHOULD_CONVERTED = 205;
aa = SHOULD_CONVERTED;
SHOULD_CONVERTED == 206;
207 == SHOULD_CONVERTED;

// just make sure `MemberExpr` fold success.

console.log(console.log(console.log));
a[b].c[d].e[f].g;

// recursive
// TODO: only
assert.equal(wurst, unde);
assert.equal(suppe, wurst);
