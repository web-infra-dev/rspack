'use strict';

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  module: {
    rules: [
      {
        test: /\.module\.css$/i,
        type: 'css/module',
      },
    ],
  },
};
