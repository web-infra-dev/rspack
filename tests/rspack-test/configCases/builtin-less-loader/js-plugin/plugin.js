module.exports = {
  install(less, _manager, functions) {
    let calls = 0;
    functions.add('from-plugin', function (value) {
      calls++;
      const factor = new less.tree.Variable('@factor').eval(this.context);
      return new less.tree.Dimension(value.value * factor.value + calls, value.unit);
    });
  },
};
