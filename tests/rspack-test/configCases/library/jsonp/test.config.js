/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  moduleScope(scope) {
    scope.MyJsonpCallback = function (exports) {
      global.__jsonpCapture = exports;
    };
  },
  afterExecute() {
    delete global.__jsonpCapture;
  },
};
