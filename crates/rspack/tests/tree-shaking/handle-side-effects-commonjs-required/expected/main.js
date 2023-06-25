(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["main"], {
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__(/* ./source */"./source/index.js");
console.log('something');
},
"./source/index.js": function (module, exports, __webpack_require__) {
var test = function test() {
    var res = new Response();
    return res;
};
var Response = function() {
    "use strict";
    function Response(mode) {
        _class_call_check(this, Response);
        // eslint-disable-next-line no-undefined
        if (mode.data === undefined) mode.data = {};
        this.data = mode.data;
        this.isMatchIgnored = false;
    }
    _create_class(Response, [
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

},function(__webpack_require__) {
var __webpack_exec__ = function(moduleId) { return __webpack_require__(__webpack_require__.s = moduleId) }
var __webpack_exports__ = (__webpack_exec__("./index.js"));

}
]);