(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__("./source/index.js");
console.log('something');
},
"./source/index.js": function (module, exports, __webpack_require__) {
var _class_call_check = __webpack_require__("../../../../../node_modules/@swc/helpers/esm/_class_call_check.js");
var _create_class = __webpack_require__("../../../../../node_modules/@swc/helpers/esm/_create_class.js");
var test = function test() {
    var res = new Response();
    return res;
};
var Response = function() {
    "use strict";
    function Response(mode) {
        _class_call_check._(this, Response);
        if (mode.data === undefined) mode.data = {};
        this.data = mode.data;
        this.isMatchIgnored = false;
    }
    _create_class._(Response, [
        {
            key: "ignoreMatch",
            value: function ignoreMatch() {
                this.isMatchIgnored = true;
            }
        }
    ]);
    return Response;
}();
var result = test();
module.exports = result;
},
"../../../../../node_modules/@swc/helpers/esm/_class_call_check.js": function (module, exports, __webpack_require__) {
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
    _class_call_check: function() {
        return _class_call_check;
    },
    _: function() {
        return _class_call_check;
    }
});
var _instanceof = __webpack_require__("../../../../../node_modules/@swc/helpers/esm/_instanceof.js");
function _class_call_check(instance, Constructor) {
    if (!_instanceof._(instance, Constructor)) throw new TypeError("Cannot call a class as a function");
}
},
"../../../../../node_modules/@swc/helpers/esm/_create_class.js": function (module, exports, __webpack_require__) {
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
    _create_class: function() {
        return _create_class;
    },
    _: function() {
        return _create_class;
    }
});
function _defineProperties(target, props) {
    for(var i = 0; i < props.length; i++){
        var descriptor = props[i];
        descriptor.enumerable = descriptor.enumerable || false;
        descriptor.configurable = true;
        if ("value" in descriptor) descriptor.writable = true;
        Object.defineProperty(target, descriptor.key, descriptor);
    }
}
function _create_class(Constructor, protoProps, staticProps) {
    if (protoProps) _defineProperties(Constructor.prototype, protoProps);
    if (staticProps) _defineProperties(Constructor, staticProps);
    return Constructor;
}
},
"../../../../../node_modules/@swc/helpers/esm/_instanceof.js": function (module, exports, __webpack_require__) {
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
    _instanceof: function() {
        return _instanceof1;
    },
    _: function() {
        return _instanceof1;
    }
});
var _instanceof = __webpack_require__("../../../../../node_modules/@swc/helpers/esm/_instanceof.js");
function _instanceof1(left, right) {
    if (right != null && typeof Symbol !== "undefined" && right[Symbol.hasInstance]) return !!right[Symbol.hasInstance](left);
    else return _instanceof._(left, right);
}
},

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__('./index.js'));

}
]);