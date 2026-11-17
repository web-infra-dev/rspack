Object.defineProperty(__rspack_context, "r", {
 configurable: true, enumerable: true,
 get: function() { return __webpack_require__; },
 set: function(value) {
  if (this === __rspack_context) __webpack_require__ = value;
  else Object.defineProperty(this, "r", { configurable: true, writable: true, value: value });
 }
});
// Native fields stay on the context. Its ordinary-module consumers can also
// see private properties that runtime extensions install on the active loader.
Object.setPrototypeOf(__rspack_context, typeof Proxy === "function" ? new Proxy({}, {
 get: function(_, key) { return __webpack_require__[key]; },
 has: function(_, key) { return key in __webpack_require__; },
 set: function(_, key, value, receiver) {
  if (Object.prototype.hasOwnProperty.call(__webpack_require__, key)) return Reflect.set(__webpack_require__, key, value);
  Object.defineProperty(receiver, key, { configurable: true, enumerable: true, writable: true, value: value });
  return true;
 }
}) : __webpack_require__);
function __rspack_bind_runtime__(key, nativeKey, get, set) {
 var descriptor = { configurable: true, enumerable: true, get: get, set: set };
 Object.defineProperty(__webpack_require__, key, descriptor);
 Object.defineProperty(__rspack_context, nativeKey, descriptor);
}
