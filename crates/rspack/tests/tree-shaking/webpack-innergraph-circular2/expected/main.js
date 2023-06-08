(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);

it("should be able to handle circular referenced", ()=>{
    expect(x()).toEqual([
        y,
        z
    ]);
    const [_a, b, c, d] = a();
    expect(b()).toEqual([
        a,
        b,
        c,
        d
    ]);
    expect(c()).toEqual([
        a,
        b,
        c,
        d
    ]);
    expect(d()).toEqual([
        a,
        b,
        c,
        d
    ]);
    const [f2, f4] = f3();
    const [f1, _f3] = f2();
    expect(_f3).toBe(f3);
    expect(f3()).toEqual(f1());
    expect(f2()).toEqual(f4());
});
},
"./module.js": function (module, exports, __webpack_require__) {
'use strict';
__webpack_require__.r(exports);
__webpack_require__.d(exports, {'x': function() { return x; }, 'y': function() { return y; }, 'z': function() { return z; }, 'a': function() { return a; }, 'f3': function() { return f3; }});
function x() {
    return [
        y,
        z
    ];
}
function y() {
    return [
        x,
        z
    ];
}
function z() {
    return [
        x,
        y
    ];
}

function a() {
    return [
        a,
        b,
        c,
        d
    ];
}
function b() {
    return [
        a,
        b,
        c,
        d
    ];
}
function c() {
    return [
        a,
        b,
        c,
        d
    ];
}
function d() {
    return [
        a,
        b,
        c,
        d
    ];
}

function f1() {
    return [
        f2,
        f4
    ];
}
function f2() {
    return [
        f1,
        f3
    ];
}
function f3() {
    return [
        f2,
        f4
    ];
}
function f4() {
    return [
        f1,
        f3
    ];
}

},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);