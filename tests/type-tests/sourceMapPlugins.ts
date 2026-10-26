import rspack, { type EvalSourceMapDevToolPluginOptions } from '@rspack/core';

const options: EvalSourceMapDevToolPluginOptions = {
  columns: true,
  debugIds: true,
  exclude: /vendor/,
  ignoreList: /node_modules/,
  include: ['src'],
  module: true,
  moduleFilenameTemplate: (info) => info.resourcePath,
  namespace: 'app',
  noSources: false,
  sourceRoot: '/',
  test: /\.js$/,
};
new rspack.EvalSourceMapDevToolPlugin(options);

// @ts-expect-error append is ignored by EvalSourceMapDevToolPlugin.
new rspack.EvalSourceMapDevToolPlugin({ append: false });
// @ts-expect-error filename is ignored by EvalSourceMapDevToolPlugin.
new rspack.EvalSourceMapDevToolPlugin({ filename: '[file].map' });
// @ts-expect-error fileContext is ignored by EvalSourceMapDevToolPlugin.
new rspack.EvalSourceMapDevToolPlugin({ fileContext: 'src' });
// @ts-expect-error publicPath is ignored by EvalSourceMapDevToolPlugin.
new rspack.EvalSourceMapDevToolPlugin({ publicPath: '/' });
new rspack.EvalSourceMapDevToolPlugin({
  // @ts-expect-error fallbackModuleFilenameTemplate is ignored by this plugin.
  fallbackModuleFilenameTemplate: '[resource-path]',
});

new rspack.SourceMapDevToolPlugin({
  ...options,
  append: false,
  filename: '[file].map',
  fileContext: 'src',
  publicPath: '/',
  fallbackModuleFilenameTemplate: '[resource-path]',
});
