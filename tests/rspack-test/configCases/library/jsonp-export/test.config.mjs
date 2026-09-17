/** @type {import("../../../..").TConfigCaseConfig} */
export default {
  moduleScope(scope) {
    scope.MyJsonpCallback = function (exports) {
      global.__jsonpExportCapture = exports;
    };
  },
  afterExecute() {
    delete global.__jsonpExportCapture;
  },
};
