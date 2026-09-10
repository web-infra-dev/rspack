const { rspack } = require('@rspack/core');

module.exports = {
  target: 'node',
  module: {
    parser: {
      javascript: {
        requireAlias: true,
        createRequire: true,
      },
    },
  },
  plugins: [
    new rspack.DefinePlugin({
      CALLER_EXPRESSION: 'require("./value")',
      LOCAL_EXPRESSION:
        '(() => { const require = () => "fragment"; return require("./missing-fragment"); })()',
      SCOPED_EXPRESSION: `(() => {
        const values = [];
        function hoisted() {
          const before = typeof require;
          var require = () => "fragment";
          return before === "undefined" ? require("./missing-hoisted") : "incorrect";
        }
        values.push(hoisted());
        values.push(((require = () => "fragment") => require("./missing-parameter"))());
        values.push((({ require } = { require: () => "fragment" }) => require("./missing-pattern"))());
        try { throw () => "fragment"; }
        catch (require) { values.push(require("./missing-catch")); }
        for (const require of [() => "fragment"]) {
          values.push(require("./missing-loop"));
        }
        switch (1) {
          case 1: {
            const require = () => "fragment";
            values.push(require("./missing-block"));
            break;
          }
        }
        const recursive = function require() {
          return typeof require === "function" ? "fragment" : "incorrect";
        };
        const Named = class require {
          static value() { return typeof require === "function" ? "fragment" : "incorrect"; }
          static {
            const require = () => "fragment";
            this.local = require("./missing-static");
          }
        };
        values.push(recursive(), Named.value(), Named.local);
        return values.every(value => value === "fragment") ? "fragment" : "incorrect";
      })()`,
      DEFAULT_EXPRESSION: `((value = require("./missing-default")) => {
        var require = () => "body";
        return value;
      })()`,
      NESTED_CONDITION: 'typeof NESTED_LITERAL === "boolean"',
      NESTED_LITERAL: 'true',
    }),
  ],
};
