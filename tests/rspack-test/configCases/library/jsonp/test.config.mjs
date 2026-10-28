/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  moduleScope(scope) {
    scope.MyJsonpCallback = function (exports) {
      global.__jsonpCapture = exports;
    };
  },
  afterExecute() {
    delete global.__jsonpCapture;
  },
};
