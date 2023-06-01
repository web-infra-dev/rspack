function _class_call_check(instance, Constructor) {
    if (!(instance instanceof Constructor)) {
        throw new TypeError("Cannot call a class as a function");
    }
}
var a = function a() {
    "use strict";
    _class_call_check(this, a);
};
