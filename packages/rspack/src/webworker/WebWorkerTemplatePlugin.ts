/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/webworker/WebWorkerTemplatePlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import {
  ArrayPushCallbackChunkFormatPlugin,
  EnableChunkLoadingPlugin,
} from '../builtin-plugin';
import type { Compiler } from '../Compiler';

export default class WebWorkerTemplatePlugin {
  apply(compiler: Compiler) {
    compiler.options.output.chunkLoading = 'import-scripts';
    new ArrayPushCallbackChunkFormatPlugin().apply(compiler);
    new EnableChunkLoadingPlugin('import-scripts').apply(compiler);
  }
}
