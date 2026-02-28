/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/util
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

export const formatSize = (size: unknown) => {
  if (typeof size !== 'number' || Number.isNaN(size)) {
    return 'unknown size';
  }

  if (size <= 0) {
    return '0 bytes';
  }

  const abbreviations = ['bytes', 'KiB', 'MiB', 'GiB'];
  const index = Math.floor(Math.log(size) / Math.log(1024));

  return `${+(size / 1024 ** index).toPrecision(3)} ${abbreviations[index]}`;
};
