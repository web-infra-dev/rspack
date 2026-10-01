import { ReadableStream } from "node:stream/web";
import { TextDecoder } from "node:util";
/** @type {import("../../../..").TConfigCaseConfig} */
export default {
    findBundle: function () {
        return ['bundle0.js'];
    },
    moduleScope(scope) {
        scope.ReadableStream = ReadableStream;
        scope.TextDecoder = TextDecoder;
    },
};
