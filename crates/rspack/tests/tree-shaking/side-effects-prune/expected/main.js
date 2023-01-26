(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"../node_modules/side-effects-module/index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "test", {
    enumerable: true,
    get: ()=>test
});
function test() {
    console.log('something');
}
},
"./app.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./src/a.js"), exports);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
const _appJs = __webpack_require__("./app.js");
_appJs.a;
},
"./src/a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: ()=>a
});
const _dJs = __webpack_require__("./src/d.js");
const a = ()=>{
    (0, _dJs.test)();
    _dJs.b;
    console.log("");
};
},
"./src/b.js": function (module, exports, __webpack_require__) {
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
    test: ()=>_cJs.test,
    b: ()=>b
});
const _cJs = __webpack_require__("./src/c.js");
const b = 3;
},
"./src/c.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "test", {
    enumerable: true,
    get: ()=>_indexJs.test
});
const _indexJs = __webpack_require__("../node_modules/side-effects-module/index.js");
},
"./src/d.js": function (module, exports, __webpack_require__) {
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
    test: ()=>_bJs.test,
    b: ()=>_bJs.b
});
const _bJs = __webpack_require__("./src/b.js");
},

},function(__webpack_require__) {
__webpack_require__("./index.js");
}
]);