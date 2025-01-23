(function (global, module, define) {

  a = 1;

  if (module && module.exports) {
    module.exports = a;
  } else if (define && define.amd) {
    define(function () { return a; });
  } else {
    this.a = a;
  }

})(
  this,
  (typeof module) == 'object' && module,
  (typeof define) == 'function' && define,
);
