// A data descriptor can replace a built-in accessor. Keep the native binding in sync.
var __rspack_runtime_keys__ = Object.create(null);
if (typeof Proxy === "function") __webpack_require__ = new Proxy(__webpack_require__, {
 defineProperty: function(target, key, descriptor) {
  if (!Reflect.defineProperty(target, key, descriptor)) return false;
  if ("value" in descriptor && key in __rspack_runtime_keys__) __rspack_context[__rspack_runtime_keys__[key]] = descriptor.value;
  return true;
 }
});
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
 get: function(_, key, receiver) {
  var loader = Object.prototype.hasOwnProperty.call(receiver, "r") ? receiver.r : __webpack_require__;
  return key in loader ? loader[key] : __webpack_require__[key];
 },
 has: function(_, key) { return key in __webpack_require__; },
 set: function(_, key, value, receiver) {
  var loader = Object.prototype.hasOwnProperty.call(receiver, "r") ? receiver.r : __webpack_require__;
  return Reflect.set(loader, key, value);
 }
}) : __webpack_require__);
function __rspack_bind_runtime__(key, nativeKey, get, set) {
 __rspack_runtime_keys__[key] = nativeKey;
 Object.defineProperty(__rspack_context, nativeKey, { configurable: true, enumerable: true, get: get, set: set });
 // Hot updates replace the context accessor; a captured loader must always use the current one.
 Object.defineProperty(__webpack_require__, key, {
  configurable: true, enumerable: true,
  get: function() { return __rspack_context[nativeKey]; },
  set: function(value) { __rspack_context[nativeKey] = value; }
 });
}
