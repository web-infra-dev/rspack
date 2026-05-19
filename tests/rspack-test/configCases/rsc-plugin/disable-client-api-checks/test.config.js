/** @type {import("../../../..").TConfigCaseConfig} */
module.exports = {
    findBundle: function () {
        return ['bundle0.js'];
    },
    moduleScope(scope) {
        scope.ReadableStream = require('stream/web').ReadableStream;
        scope.TextDecoder = require('util').TextDecoder;
    },
};
