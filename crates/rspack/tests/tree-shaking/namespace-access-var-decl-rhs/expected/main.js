(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./a.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "a", {
    enumerable: true,
    get: function() {
        return a;
    }
});
const a = {
    a: ''
};
},
"./b.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "b", {
    enumerable: true,
    get: function() {
        return b;
    }
});
const b = {
    b: ""
};
},
"./enum-old.js": function (module, exports, __webpack_require__) {
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
    a: function() {
        return _a.a;
    },
    b: function() {
        return _b.b;
    }
});
var _a = __webpack_require__("./a.js");
var _b = __webpack_require__("./b.js");
},
"./enum.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
__webpack_require__.es(__webpack_require__("./enum-old.js"), exports);
},
"./index.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
var _lib = __webpack_require__("./lib.js");
console.log(_lib.getDocPermissionTextSendMe);
},
"./lib.js": function (module, exports, __webpack_require__) {
"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "getDocPermissionTextSendMe", {
    enumerable: true,
    get: function() {
        return getDocPermissionTextSendMe;
    }
});
var _enum = __webpack_require__.ir(__webpack_require__("./enum.js"));
function Record() {}
({
    1: _enum.a.a
});
function getDocPermissionTextSendMe() {}
class Doc extends Record({}) {
    isSheet() {
        return this.type === _enum.b.b;
    }
}
Doc.fromJS = (data)=>new Doc(data);
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);