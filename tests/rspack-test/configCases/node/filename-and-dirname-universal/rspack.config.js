'use strict';

const node = {
  __dirname: 'eval-only',
  __filename: 'eval-only',
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    name: 'web',
    target: ['node', 'web'],
    node,
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    target: ['node', 'web'],
    node,
    output: {
      module: true,
    },
  },
  {
    name: 'web',
    devtool: 'eval',
    target: ['node', 'web'],
    node,
    output: {
      module: true,
    },
  },
  {
    name: 'node',
    devtool: 'eval',
    target: ['node', 'web'],
    node,
    output: {
      module: true,
    },
  },
];
