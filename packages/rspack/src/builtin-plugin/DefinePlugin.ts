import { BuiltinPluginName } from '@rspack/binding';

import { create } from './base';
import WebpackError from '../lib/WebpackError';

const IMPORT_META_ENV_KEY = 'import.meta.env';

export type DefinePluginOptions = Record<string, CodeValue>;
export const DefinePlugin = create(
  BuiltinPluginName.DefinePlugin,
  function (define: DefinePluginOptions): NormalizedCodeValue {
    const supportsBigIntLiteral =
      this.options.output.environment?.bigIntLiteral ?? false;
    const warnings: string[] = [];
    const normalizedDefine = normalizeValue(
      define,
      supportsBigIntLiteral,
      this.options.experiments.env ?? false,
      warnings,
    );
    if (warnings.length > 0) {
      this.hooks.thisCompilation.tap('DefinePlugin', (compilation) => {
        for (const warning of warnings) {
          const error = new WebpackError(warning);
          error.name = 'DefinePluginImportMetaEnvWarning';
          compilation.warnings.push(error);
        }
      });
    }
    return normalizedDefine;
  },
  'compilation',
);

const normalizeValue = (
  define: DefinePluginOptions,
  supportsBigIntLiteral: boolean,
  experimentsEnvEnabled: boolean,
  warnings: string[],
) => {
  let normalizedDefineInput: Record<string, CodeValue> = define;
  if (
    experimentsEnvEnabled &&
    Object.prototype.hasOwnProperty.call(define, IMPORT_META_ENV_KEY)
  ) {
    normalizedDefineInput = { ...define };
    normalizedDefineInput[IMPORT_META_ENV_KEY] = normalizeImportMetaEnvValue(
      define[IMPORT_META_ENV_KEY],
      warnings,
    );
  }

  const normalizePrimitive = (
    p: CodeValuePrimitive,
  ): NormalizedCodeValuePrimitive => {
    if (p === undefined) {
      return 'undefined';
    }
    if (Object.is(p, -0)) {
      return '-0';
    }
    if (p instanceof RegExp) {
      return p.toString();
    }
    if (typeof p === 'function') {
      return `(${p.toString()})`;
    }
    if (typeof p === 'bigint') {
      return supportsBigIntLiteral ? `${p}n` : `BigInt("${p}")`;
    }
    // assume `p` is a valid JSON value
    return p;
  };
  const normalizeObject = (define: CodeValue): NormalizedCodeValue => {
    if (Array.isArray(define)) {
      return define.map(normalizeObject);
    }
    if (define instanceof RegExp) {
      return normalizePrimitive(define);
    }
    if (define && typeof define === 'object') {
      const keys = Object.keys(define);
      return Object.fromEntries(
        keys.map((k) => [k, normalizeObject(define[k])]),
      );
    }
    return normalizePrimitive(define);
  };
  return normalizeObject(normalizedDefineInput);
};

const normalizeImportMetaEnvValue = (
  value: CodeValue,
  warnings: string[],
): CodeValue => {
  if (isImportMetaEnvObject(value)) {
    return value;
  }
  if (typeof value !== 'string') {
    warnings.push(
      'DefinePlugin: the value of "import.meta.env" should be an object or a JSON stringified object.',
    );
    return {};
  }

  try {
    if (!isImportMetaEnvObject(JSON.parse(value))) {
      throw new TypeError('Expected an object');
    }
  } catch {
    warnings.push(
      'DefinePlugin: the string value of "import.meta.env" should be a JSON stringified object.',
    );
    return {};
  }
  return value;
};

const isImportMetaEnvObject = (
  value: unknown,
): value is Record<string, CodeValue> =>
  value !== null &&
  typeof value === 'object' &&
  !Array.isArray(value) &&
  !(value instanceof RegExp);

type CodeValue = RecursiveArrayOrRecord<CodeValuePrimitive>;
type CodeValuePrimitive =
  null | undefined | RegExp | Function | string | number | boolean | bigint;
type NormalizedCodeValuePrimitive = null | string | number | boolean;
type NormalizedCodeValue = RecursiveArrayOrRecord<NormalizedCodeValuePrimitive>;

type RecursiveArrayOrRecord<T> =
  | { [index: string]: RecursiveArrayOrRecord<T> }
  | RecursiveArrayOrRecord<T>[]
  | T;
