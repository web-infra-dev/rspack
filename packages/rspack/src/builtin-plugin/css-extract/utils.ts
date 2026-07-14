import path from 'node:path';

import type { Compiler, LoaderContext } from '../..';

// the plugin and the loader are bundled separately, so the marker for
// `runtime: false` compilers lives on the compiler object under a global
// registry symbol: they get no hmrC.miniCss handler and the hot loader
// must keep its own change signal
const CSS_RUNTIME_DISABLED = Symbol.for(
  'rspack.CssExtractRspackPlugin.runtimeDisabled',
);

export function markCssRuntimeDisabled(compiler: Compiler) {
  (compiler as { [CSS_RUNTIME_DISABLED]?: boolean })[CSS_RUNTIME_DISABLED] =
    true;
}

export function isCssRuntimeDisabled(compiler: Compiler | undefined) {
  return (
    !!compiler &&
    (compiler as { [CSS_RUNTIME_DISABLED]?: boolean })[CSS_RUNTIME_DISABLED] ===
      true
  );
}

export function isAbsolutePath(str: string) {
  return path.posix.isAbsolute(str) || path.win32.isAbsolute(str);
}

export const PLUGIN_NAME = 'css-extract-rspack-plugin';

const RELATIVE_PATH_REGEXP = /^\.\.?[/\\]/;

export function isRelativePath(str: string) {
  return RELATIVE_PATH_REGEXP.test(str);
}

export function stringifyRequest(
  loaderContext: LoaderContext,
  request: string,
) {
  if (
    typeof loaderContext.utils !== 'undefined' &&
    typeof loaderContext.utils.contextify === 'function'
  ) {
    return JSON.stringify(
      loaderContext.utils.contextify(
        loaderContext.context || loaderContext.rootContext,
        request,
      ),
    );
  }

  const splitted = request.split('!');
  const { context } = loaderContext;

  return JSON.stringify(
    splitted
      .map((part) => {
        // First, separate singlePath from query, because the query might contain paths again
        const splittedPart = part.match(/^(.*?)(\?.*)/);
        const query = splittedPart ? splittedPart[2] : '';
        let singlePath = splittedPart ? splittedPart[1] : part;

        if (isAbsolutePath(singlePath) && context) {
          singlePath = path.relative(context, singlePath);

          if (isAbsolutePath(singlePath)) {
            // If singlePath still matches an absolute path, singlePath was on a different drive than context.
            // In this case, we leave the path platform-specific without replacing any separators.
            // @see https://github.com/webpack/loader-utils/pull/14
            return singlePath + query;
          }

          if (!isRelativePath(singlePath)) {
            // Ensure that the relative path starts at least with ./ otherwise it would be a request into the modules directory (like node_modules).
            singlePath = `./${singlePath}`;
          }
        }

        return singlePath.replace(/\\/g, '/') + query;
      })
      .join('!'),
  );
}

export function stringifyLocal(value: any) {
  return typeof value === 'function' ? value.toString() : JSON.stringify(value);
}
