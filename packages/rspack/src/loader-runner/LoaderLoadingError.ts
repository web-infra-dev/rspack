/**
 * The following code is from
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */

class LoadingLoaderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'LoaderRunnerError';
    Error.captureStackTrace(this, this.constructor);
  }
}

export default LoadingLoaderError;
