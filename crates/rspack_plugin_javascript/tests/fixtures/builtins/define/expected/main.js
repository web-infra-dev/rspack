(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _libJs = __webpack_require__.ir(__webpack_require__("./lib.js"));
const lib = __webpack_require__("./lib.js");
const { DO_NOT_CONVERTED9  } = __webpack_require__("./lib.js");
equal(true, true);
assert.deepStrictEqual(5, 5);
assert.deepStrictEqual(null, null);
assert.deepStrictEqual(undefined, undefined);
assert.deepStrictEqual(100.05, 100.05);
assert.deepStrictEqual(0, 0);
let ZERO_OBJ = {
    ZERO: 0
};
assert.deepStrictEqual(ZERO_OBJ.ZERO, 0);
assert.deepStrictEqual(ZERO_OBJ[0], undefined);
assert.deepStrictEqual(ZERO_OBJ[0], undefined);
assert.deepStrictEqual(ZERO_OBJ["ZERO"], 0);
assert.deepStrictEqual(BigInt(10000), 10000n);
assert.deepStrictEqual(100000000000n, 100000000000n);
assert.deepStrictEqual(0, 0);
assert.deepStrictEqual(-0, -0);
assert.deepStrictEqual(100.25, 100.25);
assert.deepStrictEqual(-100.25, -100.25);
assert.deepStrictEqual("string", "string");
assert.deepStrictEqual("", "");
assert.deepStrictEqual(/abc/i, /abc/i);
assert.deepStrictEqual(0..ABC, undefined);
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
} catch (err1) {}
assert.deepStrictEqual(error_count, 2);
assert.deepStrictEqual([
    300,
    [
        "six"
    ]
], [
    300,
    [
        "six"
    ]
]);
assert.deepStrictEqual(300, 300);
assert.deepStrictEqual(300[1], undefined);
assert.deepStrictEqual([
    "six"
], [
    "six"
]);
assert.deepStrictEqual([
    "six"
][0], "six");
assert.deepStrictEqual([
    "six"
][0][0], "s");
assert.deepStrictEqual([
    "six"
], [
    "six"
]);
assert.deepStrictEqual([
    300,
    [
        "six"
    ]
][[
    300,
    [
        "six"
    ]
]], undefined);
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}, {
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
});
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.OBJ, {
    NUM: 1
});
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.OBJ.NUM, 1);
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.UNDEFINED, undefined);
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.REGEXP, /def/i);
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.STR, "string");
assert.deepStrictEqual({
    UNDEFINED: undefined,
    REGEXP: /def/i,
    STR: "string",
    OBJ: {
        NUM: 1
    }
}.AAA, undefined);
assert.deepStrictEqual(301, 301);
assert.deepStrictEqual("302", "302");
assert.deepStrictEqual(303, 303);
assert.deepStrictEqual(304, 304);
assert.deepStrictEqual(303..P4, undefined);
try {
    error_count += 1;
    P4.P1;
    error_count += 1;
} catch (err2) {}
assert.deepStrictEqual(error_count, 3);
assert.deepStrictEqual("302".P1, undefined);
assert.deepStrictEqual("302".P3, undefined);
assert.deepStrictEqual("302".P4, undefined);
const DO_NOT_CONVERTED = 201;
assert.deepStrictEqual(DO_NOT_CONVERTED, 201);
let { DO_NOT_CONVERTED2  } = {
    DO_NOT_CONVERTED2: 202
};
assert.deepStrictEqual(DO_NOT_CONVERTED2, 202);
const { c: DO_NOT_CONVERTED3  } = {
    c: 203
};
assert.deepStrictEqual(DO_NOT_CONVERTED3, 203);
try {
    error_count += 1;
    DO_NOT_CONVERTED4;
    error_count += 1;
} catch (err3) {}
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
    const B = 0;
    assert.deepStrictEqual(B, 0);
    let IN_BLOCK = 2;
    assert.deepStrictEqual(IN_BLOCK, 2);
    assert.deepStrictEqual(205, 205);
}try {
    error_count += 1;
    SHOULD_BE_CONVERTED_IN_UNDEFINED_BLOCK;
    error_count += 1;
} catch (err4) {}
assert.deepStrictEqual(error_count, 5);
assert.deepStrictEqual(USELESS, {
    ZERO: 0
});
assert.deepStrictEqual({}.DO_NOT_CONVERTED5, undefined);
assert.deepStrictEqual({}.DO_NOT_CONVERTED6, undefined);
assert.deepStrictEqual(_libJs.DO_NOT_CONVERTED7, 402);
assert.deepStrictEqual(_libJs.default, 401);
assert.deepStrictEqual(DO_NOT_CONVERTED9, 403);
assert.deepStrictEqual(lib.DO_NOT_CONVERTED9, 403);
try {
    error_count += 1;
    M1;
    error_count += 1;
} catch (err5) {}
assert.deepStrictEqual(error_count, 6);
try {
    error_count += 1;
    aa = 205;
    error_count += 1;
} catch (err6) {}
assert.deepStrictEqual(error_count, 7);
assert.deepStrictEqual(true, true);
assert.deepStrictEqual(false, false);
try {
    error_count += 1;
    A1.A2.A3;
    error_count += 1;
} catch (err7) {}
assert.deepStrictEqual(error_count, 8);
console.log(console.log(console.log));
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    default: function() {
        return _default;
    },
    DO_NOT_CONVERTED7: function() {
        return DO_NOT_CONVERTED7;
    },
    DO_NOT_CONVERTED9: function() {
        return DO_NOT_CONVERTED9;
    }
});
const DO_NOT_CONVERTED7 = 402;
const DO_NOT_CONVERTED9 = 403;
var _default = 401;
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);