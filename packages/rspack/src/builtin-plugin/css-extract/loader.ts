import path from 'node:path';

import type { Filename, LoaderContext, LoaderDefinition } from '../..';
import { PLUGIN_NAME, stringifyLocal, stringifyRequest } from './utils';

export const BASE_URI = 'rspack-css-extract://';
export const MODULE_TYPE = 'css/mini-extract';
export const AUTO_PUBLIC_PATH = '__css_extract_public_path_auto__';
export const ABSOLUTE_PUBLIC_PATH = `${BASE_URI}/css-extract-plugin/`;
export const SINGLE_DOT_PATH_SEGMENT =
  '__css_extract_single_dot_path_segment__';

interface DependencyDescription {
  identifier: string;
  content: string;
  context: string;
  media?: string;
  supports?: string;
  layer?: string;
  sourceMap?: string;
  identifierIndex: number;
  filepath: string;
}

export interface CssExtractRspackLoaderOptions {
  publicPath?: string | ((resourcePath: string, context: string) => string);
  emit?: boolean;
  esModule?: boolean;
  layer?: string;
  defaultExport?: boolean;
}

export function hotLoader(
  content: string,
  context: {
    loaderContext: LoaderContext;
    options?: CssExtractRspackLoaderOptions;
    locals?: Record<string, string>;
  },
): string {
  const localsJsonString = JSON.stringify(JSON.stringify(context.locals));
  return `${content}
    if(module.hot) {
      (function() {
        var localsJsonString = ${localsJsonString};
        // ${Date.now()}
        var cssReload = require(${stringifyRequest(
          context.loaderContext,
          path.join(__dirname, 'cssExtractHmr.js'),
        )}).cssReload(module.id, ${JSON.stringify(context.options ?? {})});
        // only invalidate when locals change
        if (
          module.hot.data &&
          module.hot.data.value &&
          module.hot.data.value !== localsJsonString
        ) {
          module.hot.invalidate();
        } else {
          module.hot.accept();
        }
        module.hot.dispose(function(data) {
          data.value = localsJsonString;
          cssReload();
        });
      })();
    }
  `;
}

const loader: LoaderDefinition = function loader(content) {
  if (
    this._compiler?.options?.experiments?.css &&
    this._module &&
    (this._module.type === 'css' ||
      this._module.type === 'css/auto' ||
      this._module.type === 'css/global' ||
      this._module.type === 'css/module')
  ) {
    return content;
  }
};

export const pitch: LoaderDefinition['pitch'] = function (request, _, data) {
  if (
    this._compiler?.options?.experiments?.css &&
    this._module &&
    (this._module.type === 'css' ||
      this._module.type === 'css/auto' ||
      this._module.type === 'css/global' ||
      this._module.type === 'css/module')
  ) {
    const e = new Error(
      `use type 'css' and \`CssExtractRspackPlugin\` together, please set \`experiments.css\` to \`false\` or set \`{ type: "javascript/auto" }\` for rules with \`CssExtractRspackPlugin\` in your rspack config (now \`CssExtractRspackPlugin\` does nothing).`,
    );
    e.stack = undefined;
    this.emitWarning(e);

    return;
  }

  const options = this.getOptions() as CssExtractRspackLoaderOptions;
  const emit = typeof options.emit !== 'undefined' ? options.emit : true;
  const callback = this.async();
  const filepath = this.resourcePath;

  this.addDependency(filepath);

  let { publicPath } = this._compilation.outputOptions;

  if (typeof options.publicPath === 'string') {
    // eslint-disable-next-line prefer-destructuring
    publicPath = options.publicPath;
  } else if (typeof options.publicPath === 'function') {
    publicPath = options.publicPath(this.resourcePath, this.rootContext);
  }

  if (publicPath === 'auto') {
    publicPath = AUTO_PUBLIC_PATH;
  }

  let publicPathForExtract: Filename | undefined;

  if (typeof publicPath === 'string') {
    const isAbsolutePublicPath = /^[a-zA-Z][a-zA-Z\d+\-.]*?:/.test(publicPath);

    publicPathForExtract = isAbsolutePublicPath
      ? publicPath
      : `${ABSOLUTE_PUBLIC_PATH}${publicPath.replace(
          /\./g,
          SINGLE_DOT_PATH_SEGMENT,
        )}`;
  } else {
    publicPathForExtract = publicPath;
  }

  const handleExports = (
    originalExports:
      | { default: Record<string, any>; __esModule: true }
      | Record<string, any>,
  ) => {
    let locals: Record<string, string> | undefined;
    let namedExport: boolean;

    const esModule =
      typeof options.esModule !== 'undefined' ? options.esModule : true;
    let dependencies: DependencyDescription[] = [];

    try {
      // eslint-disable-next-line no-underscore-dangle
      const exports = originalExports.__esModule
        ? originalExports.default
        : originalExports;

      namedExport =
        // eslint-disable-next-line no-underscore-dangle
        originalExports.__esModule &&
        (!originalExports.default || !('locals' in originalExports.default));

      if (namedExport) {
        for (const key of Object.keys(originalExports)) {
          if (key !== 'default') {
            if (!locals) {
              locals = {};
            }

            locals[key] = (originalExports as Record<string, string>)[key];
          }
        }
      } else {
        locals = exports?.locals;
      }

      if (Array.isArray(exports) && emit) {
        const identifierCountMap = new Map();

        dependencies = exports
          .map(([id, content, media, sourceMap, supports, layer]) => {
            const identifier = id;
            const context = this.rootContext;

            const count = identifierCountMap.get(identifier) || 0;

            identifierCountMap.set(identifier, count + 1);

            return {
              identifier,
              context,
              content,
              media,
              supports,
              layer,
              identifierIndex: count,
              sourceMap: sourceMap
                ? JSON.stringify(sourceMap)
                : // eslint-disable-next-line no-undefined
                  undefined,
              filepath,
            };
          })
          .filter((item) => item !== null) as DependencyDescription[];
      }
    } catch (e) {
      callback(e as Error);

      return;
    }

    const result = (function makeResult() {
      if (locals) {
        if (namedExport) {
          const identifiers = Array.from(
            (function* generateIdentifiers() {
              let identifierId = 0;

              for (const key of Object.keys(locals)) {
                identifierId += 1;

                yield [`_${identifierId.toString(16)}`, key];
              }
            })(),
          );

          const localsString = identifiers
            .map(([id, key]) => `\nvar ${id} = ${stringifyLocal(locals[key])};`)
            .join('');
          const exportsString = `export { ${identifiers
            .map(([id, key]) => `${id} as ${JSON.stringify(key)}`)
            .join(', ')} }`;

          const defaultExport =
            typeof options.defaultExport !== 'undefined'
              ? options.defaultExport
              : false;

          return defaultExport
            ? `${localsString}\n${exportsString}\nexport default { ${identifiers
                .map(([id, key]) => `${JSON.stringify(key)}: ${id}`)
                .join(', ')} }\n`
            : `${localsString}\n${exportsString}\n`;
        }

        return `\n${
          esModule ? 'export default' : 'module.exports = '
        } ${JSON.stringify(locals)};`;
      }
      if (esModule) {
        return '\nexport {};';
      }
      return '';
    })();

    let resultSource = `// extracted by ${PLUGIN_NAME}`;

    // only attempt hotreloading if the css is actually used for something other than hash values
    resultSource +=
      this.hot && emit
        ? hotLoader(result, { loaderContext: this, options, locals: locals! })
        : result;

    if (dependencies.length > 0) {
      this.__internal__setParseMeta(PLUGIN_NAME, JSON.stringify(dependencies));
    }

    callback(null, resultSource, undefined, data);
  };

  this.importModule(
    `${this.resourcePath}.rspack[javascript/auto]!=!!!${request}`,
    {
      layer: options.layer,
      publicPath: publicPathForExtract,
      baseUri: `${BASE_URI}/`,
    },
    (error, exports) => {
      if (error) {
        callback(error);

        return;
      }

      handleExports(exports);
    },
  );
};

export default loader;
