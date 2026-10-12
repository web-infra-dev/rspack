import { defineConfig } from '@rspack/cli';
import { rspack } from '@rspack/core';

const lineTerminators = [
  { name: 'lf', separator: '\n', original: 'line-lf-original.js' },
  { name: 'crlf', separator: '\r\n', original: 'line-crlf-original.js' },
  { name: 'cr', separator: '\r', original: 'line-cr-original.js' },
  { name: 'ls', separator: '\u2028', original: 'line-ls-original.js' },
  { name: 'ps', separator: '\u2029', original: 'line-ps-original.js' },
];

const lineTerminatorModules = Object.fromEntries(
  lineTerminators.flatMap(({ name, separator, original }) => [
    [
      `line-${name}.js`,
      [
        `export const value = () => "${name}";`,
        `//  #  sourceMappingURL  =  line-${name}.js.map`,
        '// ordinary trailing comment',
        '\t ',
      ].join(separator),
    ],
    [
      `line-${name}.js.map`,
      JSON.stringify({
        version: 3,
        sources: [original],
        sourcesContent: ['export const value = () => "original";'],
        names: [],
        mappings: 'AAAA',
      }),
    ],
  ]),
);

export default defineConfig({
  target: 'node',
  devtool: 'source-map',
  module: {
    rules: [
      { extractSourceMap: true },
      { test: /\.css$/, type: 'css', generator: { exportsOnly: false } },
    ],
  },
  plugins: [new rspack.experiments.VirtualModulesPlugin(lineTerminatorModules)],
});
