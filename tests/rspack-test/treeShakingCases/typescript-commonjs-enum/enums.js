'use strict';
Object.defineProperty(exports, '__esModule', { value: true });
exports.sideEffectCount = exports.readInternal = exports.Impure = exports.UnusedString = exports.Unused = exports.Used = void 0;

var Used;
(function (Used) {
  Used[Used['One'] = 1] = 'One';
})(Used || (exports.Used = Used = {}));

var Unused;
(function (Unused) {
  Unused[Unused['One'] = 1] = 'One';
  Unused[Unused['Two'] = 2] = 'Two';
})(Unused || (exports.Unused = Unused = {}));

var UnusedString;
(function (UnusedString) {
  UnusedString['Value'] = 'unused';
})(UnusedString || (exports.UnusedString = UnusedString = {}));

var Internal;
(function (Internal) {
  Internal['Value'] = 'internal';
})(Internal || (exports.Internal = Internal = {}));
exports.readInternal = () => Internal.Value;

var calls = 0;
function sideEffect() {
  calls++;
  return 1;
}
var Impure;
(function (Impure) {
  Impure[Impure['Value'] = sideEffect()] = 'Value';
})(Impure || (exports.Impure = Impure = {}));
exports.sideEffectCount = () => calls;
