/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/Template.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

const START_LOWERCASE_ALPHABET_CODE = 'a'.charCodeAt(0);
const START_UPPERCASE_ALPHABET_CODE = 'A'.charCodeAt(0);
const DELTA_A_TO_Z = 'z'.charCodeAt(0) - START_LOWERCASE_ALPHABET_CODE + 1;
const NUMBER_OF_IDENTIFIER_START_CHARS = DELTA_A_TO_Z * 2 + 2; // a-z A-Z _ $
const NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS =
  NUMBER_OF_IDENTIFIER_START_CHARS + 10; // a-z A-Z _ $ 0-9
const FUNCTION_CONTENT_REGEX = /^function\s?\(\)\s?\{\r?\n?|\r?\n?\}$/g;
const INDENT_MULTILINE_REGEX = /^\t/gm;
const LINE_SEPARATOR_REGEX = /\r?\n/g;
const IDENTIFIER_NAME_REPLACE_REGEX = /^([^a-zA-Z$_])/;
const IDENTIFIER_ALPHA_NUMERIC_NAME_REPLACE_REGEX = /[^a-zA-Z0-9$]+/g;
const COMMENT_END_REGEX = /\*\//g;
const PATH_NAME_NORMALIZE_REPLACE_REGEX = /[^a-zA-Z0-9_!§$()=\-^°]+/g;
const MATCH_PADDED_HYPHENS_REPLACE_REGEX = /^-|-$/g;

class Template {
  /**
   *
   * @param fn a runtime function (.runtime.js) "template"
   * @returns the updated and normalized function string
   */
  static getFunctionContent(fn: Function): string {
    return fn
      .toString()
      .replace(FUNCTION_CONTENT_REGEX, '')
      .replace(INDENT_MULTILINE_REGEX, '')
      .replace(LINE_SEPARATOR_REGEX, '\n');
  }

  /**
   * @param str the string converted to identifier
   * @returns created identifier
   */
  static toIdentifier(str: any): string {
    if (typeof str !== 'string') return '';
    return str
      .replace(IDENTIFIER_NAME_REPLACE_REGEX, '_$1')
      .replace(IDENTIFIER_ALPHA_NUMERIC_NAME_REPLACE_REGEX, '_');
  }
  /**
   *
   * @param str string to be converted to commented in bundle code
   * @returns returns a commented version of string
   */
  static toComment(str: string): string {
    if (!str) return '';
    return `/*! ${str.replace(COMMENT_END_REGEX, '* /')} */`;
  }

  /**
   *
   * @param str string to be converted to "normal comment"
   * @returns returns a commented version of string
   */
  static toNormalComment(str: string): string {
    if (!str) return '';
    return `/* ${str.replace(COMMENT_END_REGEX, '* /')} */`;
  }

  /**
   * @param str string path to be normalized
   * @returns normalized bundle-safe path
   */
  static toPath(str: string): string {
    if (typeof str !== 'string') return '';
    return str
      .replace(PATH_NAME_NORMALIZE_REPLACE_REGEX, '-')
      .replace(MATCH_PADDED_HYPHENS_REPLACE_REGEX, '');
  }

  // map number to a single character a-z, A-Z or multiple characters if number is too big
  /**
   * @param num number to convert to ident
   * @returns returns single character ident
   */
  static numberToIdentifier(num: number): string {
    let n = num;

    if (n >= NUMBER_OF_IDENTIFIER_START_CHARS) {
      // use multiple letters
      return (
        Template.numberToIdentifier(n % NUMBER_OF_IDENTIFIER_START_CHARS) +
        Template.numberToIdentifierContinuation(
          Math.floor(n / NUMBER_OF_IDENTIFIER_START_CHARS),
        )
      );
    }

    // lower case
    if (n < DELTA_A_TO_Z) {
      return String.fromCharCode(START_LOWERCASE_ALPHABET_CODE + n);
    }
    n -= DELTA_A_TO_Z;

    // upper case
    if (n < DELTA_A_TO_Z) {
      return String.fromCharCode(START_UPPERCASE_ALPHABET_CODE + n);
    }

    if (n === DELTA_A_TO_Z) return '_';
    return '$';
  }

  /**
   * @param num number to convert to ident
   * @returns returns single character ident
   */
  static numberToIdentifierContinuation(num: number): string {
    let n = num;
    if (n >= NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS) {
      // use multiple letters
      return (
        Template.numberToIdentifierContinuation(
          n % NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
        ) +
        Template.numberToIdentifierContinuation(
          Math.floor(n / NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS),
        )
      );
    }

    // lower case
    if (n < DELTA_A_TO_Z) {
      return String.fromCharCode(START_LOWERCASE_ALPHABET_CODE + n);
    }
    n -= DELTA_A_TO_Z;

    // upper case
    if (n < DELTA_A_TO_Z) {
      return String.fromCharCode(START_UPPERCASE_ALPHABET_CODE + n);
    }
    n -= DELTA_A_TO_Z;

    // numbers
    if (n < 10) {
      return `${n}`;
    }

    if (n === 10) return '_';
    return '$';
  }

  /**
   *
   * @param s string to convert to identity
   * @returns converted identity
   */
  static indent(s: string | string[]): string {
    if (Array.isArray(s)) {
      return s.map(Template.indent).join('\n');
    }
    const str = s.trimEnd();
    if (!str) return '';
    const ind = str[0] === '\n' ? '' : '\t';
    return ind + str.replace(/\n([^\n])/g, '\n\t$1');
  }

  /**
   *
   * @param s string to create prefix for
   * @param prefix prefix to compose
   * @returns returns new prefix string
   */
  static prefix(s: string | string[], prefix: string): string {
    const str = Template.asString(s).trim();
    if (!str) return '';
    const ind = str[0] === '\n' ? '' : prefix;
    return ind + str.replace(/\n([^\n])/g, `\n${prefix}$1`);
  }

  /**
   *
   * @param str string or string collection
   * @returns returns a single string from array
   */
  static asString(str: string | string[]): string {
    if (Array.isArray(str)) {
      return str.join('\n');
    }
    return str;
  }

  /**
   * @param modules a collection of modules to get array bounds for
   * @returns returns the upper and lower array bounds
   * or false if not every module has a number based id
   */
  static getModulesArrayBounds(
    modules: { id: string | number }[],
  ): [number, number] | false {
    let maxId = Number.NEGATIVE_INFINITY;
    let minId = Number.POSITIVE_INFINITY;
    for (const module of modules) {
      const moduleId = module.id;
      if (typeof moduleId !== 'number') return false;
      if (maxId < moduleId) maxId = moduleId;
      if (minId > moduleId) minId = moduleId;
    }
    if (minId < 16 + `${minId}`.length) {
      // add minId x ',' instead of 'Array(minId).concat(…)'
      minId = 0;
    }
    // start with -1 because the first module needs no comma
    let objectOverhead = -1;
    for (const module of modules) {
      // module id + colon + comma
      objectOverhead += `${module.id}`.length + 2;
    }
    // number of commas, or when starting non-zero the length of Array(minId).concat()
    const arrayOverhead = minId === 0 ? maxId : 16 + `${minId}`.length + maxId;
    return arrayOverhead < objectOverhead ? [minId, maxId] : false;
  }
}

export { Template };
