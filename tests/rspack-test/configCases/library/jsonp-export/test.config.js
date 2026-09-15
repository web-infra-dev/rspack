/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
  moduleScope(scope) {
    scope.MyJsonpCallback = function (exports) {
      global.__jsonpExportCapture = exports;
    };
  },
  afterExecute() {
    delete global.__jsonpExportCapture;
  },
};
