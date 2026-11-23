module.exports = {
  fn() {
    return function mockFn() {};
  },
  hoisted(factory) {
    return factory();
  },
};
