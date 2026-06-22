var exports = {
  set foo(value) {
    console.log(value);
  },
};

exports.foo = "keep shadowed exports write side effect";

var module = {
  exports: {
    set bar(value) {
      console.log(value);
    },
  },
};

module.exports.bar = "keep shadowed module write side effect";
