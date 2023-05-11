self['hotUpdate']('main', {
"./a.js": function (module, exports, __webpack_require__) {
console.log('b');
},
"./index.js": function (module, exports, __webpack_require__) {
__webpack_require__("./a.js");
__webpack_require__("./d.js");
},

});