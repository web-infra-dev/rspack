exports.foo = {
  set bar(value) {
    console.log(value);
  },
};

exports.foo.bar = "keep nested write side effect";

module.exports.baz = {
  set qux(value) {
    console.log(value);
  },
};

module.exports.baz.qux = "keep module nested write side effect";
