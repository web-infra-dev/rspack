module.exports = {
  set foo(value) {
    console.log(value);
  },
};

module.exports.foo = "keep reassigned module exports write side effect";
