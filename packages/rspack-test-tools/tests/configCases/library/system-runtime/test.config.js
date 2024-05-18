const System = require("../../../../dist/helper/legacy/fakeSystem");

/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  beforeExecute: () => {
    System.init();
  },
  findBundle() {
    return ["./main.js"];
  },
  moduleScope(scope) {
    System.setRequire(scope.require);
    scope.System = System;
  },
  afterExecute: () => {
    System.execute("(anonym)");
  }
};
