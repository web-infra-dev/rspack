/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/v5.101.0/lib/RuntimeTemplate.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import type { Compilation } from './Compilation';
import { Template } from './Template';
import type { Environment, OutputNormalized } from './config';

const SAFE_IDENTIFIER = /^[_a-zA-Z$][_a-zA-Z$0-9]*$/;
const RESERVED_IDENTIFIER = new Set([
  'break',
  'case',
  'catch',
  'class',
  'const',
  'continue',
  'debugger',
  'default',
  'delete',
  'do',
  'else',
  'export',
  'extends',
  'finally',
  'for',
  'function',
  'if',
  'import',
  'in',
  'instanceof',
  'new',
  'return',
  'super',
  'switch',
  'this',
  'throw',
  'try',
  'typeof',
  'var',
  'void',
  'while',
  'with',
  'enum',
  // strict mode
  'implements',
  'interface',
  'let',
  'package',
  'private',
  'protected',
  'public',
  'static',
  'yield',
  // module code
  'await',
  // skip future reserved keywords defined under ES1 till ES3
  // additional
  'null',
  'true',
  'false',
]);

const propertyAccess = (properties: string[], start = 0): string => {
  let str = '';
  for (let i = start; i < properties.length; i++) {
    const p = properties[i];
    if (`${+p}` === p) {
      str += `[${p}]`;
    } else if (SAFE_IDENTIFIER.test(p) && !RESERVED_IDENTIFIER.has(p)) {
      str += `.${p}`;
    } else {
      str += `[${JSON.stringify(p)}]`;
    }
  }
  return str;
};

const getGlobalObject = (
  definition: string | undefined,
): string | undefined => {
  if (!definition) return definition;
  const trimmed = definition.trim();

  if (
    // identifier, we do not need real identifier regarding ECMAScript/Unicode
    /^[_\p{L}][_0-9\p{L}]*$/iu.test(trimmed) ||
    // iife
    // call expression
    // expression in parentheses
    /^([_\p{L}][_0-9\p{L}]*)?\(.*\)$/iu.test(trimmed)
  ) {
    return trimmed;
  }

  return `Object(${trimmed})`;
};

export type ConcatenationArg = string | { expr: string };

/**
 * Helpers to generate runtime code that matches the capabilities of the
 * targeted environment, available as `compilation.runtimeTemplate`.
 *
 * Mirrors webpack's `RuntimeTemplate`, and the environment dependent branches
 * behave the same as `RuntimeTemplate` in
 * `crates/rspack_core/src/runtime_template.rs`.
 *
 * Note: the webpack methods that render references to other modules
 * (`moduleId`, `moduleRaw`, `moduleExports`, `moduleNamespace`,
 * `moduleNamespacePromise`, `importStatement`, `exportFromImport`,
 * `blockPromise`, `asyncModuleFactory`, `syncModuleFactory`, `weakError`,
 * `runtimeConditionExpression`) are not part of this class. They need read
 * access to the module graph and the code generation results, which Rspack
 * keeps on the Rust side. `comment()` is omitted because Rspack has no
 * `compilation.requestShortener`, and `defineEsModuleFlagStatement()` because
 * Rspack renders runtime globals from a numeric `RuntimeGlobals` enum whose
 * final identifier depends on the runtime mode.
 */
export class RuntimeTemplate {
  compilation: Compilation;
  outputOptions: OutputNormalized;
  globalObject: string | undefined;
  contentHashReplacement: string;

  constructor(compilation: Compilation, outputOptions: OutputNormalized) {
    this.compilation = compilation;
    this.outputOptions = outputOptions || {};
    this.globalObject = getGlobalObject(this.outputOptions.globalObject);
    this.contentHashReplacement = 'X'.repeat(
      this.outputOptions.hashDigestLength ?? 0,
    );
  }

  get #environment(): Environment {
    return this.outputOptions.environment ?? {};
  }

  isIIFE(): boolean | undefined {
    return this.outputOptions.iife;
  }

  isModule(): boolean | undefined {
    return this.outputOptions.module;
  }

  isNeutralPlatform(): boolean {
    return (
      !this.#environment.document && !this.compilation.compiler.platform.node
    );
  }

  supportsConst(): boolean {
    return Boolean(this.#environment.const);
  }

  supportsArrowFunction(): boolean {
    return Boolean(this.#environment.arrowFunction);
  }

  supportsAsyncFunction(): boolean {
    return Boolean(this.#environment.asyncFunction);
  }

  supportsOptionalChaining(): boolean {
    return Boolean(this.#environment.optionalChaining);
  }

  supportsForOf(): boolean {
    return Boolean(this.#environment.forOf);
  }

  supportsDestructuring(): boolean {
    return Boolean(this.#environment.destructuring);
  }

  supportsBigIntLiteral(): boolean {
    return Boolean(this.#environment.bigIntLiteral);
  }

  supportsDynamicImport(): boolean {
    return Boolean(this.#environment.dynamicImport);
  }

  supportsEcmaScriptModuleSyntax(): boolean {
    return Boolean(this.#environment.module);
  }

  supportTemplateLiteral(): boolean {
    return Boolean(this.#environment.templateLiteral);
  }

  supportNodePrefixForCoreModules(): boolean {
    return Boolean(this.#environment.nodePrefixForCoreModules);
  }

  /**
   * @param mod a module
   * @returns a module with `node:` prefix when supported, otherwise an original name
   */
  renderNodePrefixForCoreModule(mod: string): string {
    return this.supportNodePrefixForCoreModules()
      ? `"node:${mod}"`
      : `"${mod}"`;
  }

  /**
   * @param returnValue return value
   * @param args arguments
   * @returns returning function
   */
  returningFunction(returnValue: string, args = ''): string {
    return this.supportsArrowFunction()
      ? `(${args}) => (${returnValue})`
      : `function(${args}) { return ${returnValue}; }`;
  }

  /**
   * @param args arguments
   * @param body body
   * @returns basic function
   */
  basicFunction(args: string, body: string | string[]): string {
    return this.supportsArrowFunction()
      ? `(${args}) => {\n${Template.indent(body)}\n}`
      : `function(${args}) {\n${Template.indent(body)}\n}`;
  }

  /**
   * @param args args
   * @returns result expression
   */
  concatenation(...args: ConcatenationArg[]): string {
    const len = args.length;

    if (len === 2) return this.#es5Concatenation(args);
    if (len === 0) return '""';
    if (len === 1) {
      return typeof args[0] === 'string'
        ? JSON.stringify(args[0])
        : `"" + ${args[0].expr}`;
    }
    if (!this.supportTemplateLiteral()) return this.#es5Concatenation(args);

    // cost comparison between template literal and concatenation:
    // both need equal surroundings: `xxx` vs "xxx"
    // template literal has constant cost of 3 chars for each expression
    // es5 concatenation has cost of 3 + n chars for n expressions in row
    // when a es5 concatenation ends with an expression it reduces cost by 3
    // when a es5 concatenation starts with an single expression it reduces cost by 3
    // e. g. `${a}${b}${c}` (3*3 = 9) is longer than ""+a+b+c ((3+3)-3 = 3)
    // e. g. `x${a}x${b}x${c}x` (3*3 = 9) is shorter than "x"+a+"x"+b+"x"+c+"x" (4+4+4 = 12)

    let templateCost = 0;
    let concatenationCost = 0;

    let lastWasExpr = false;
    for (const arg of args) {
      const isExpr = typeof arg !== 'string';
      if (isExpr) {
        templateCost += 3;
        concatenationCost += lastWasExpr ? 1 : 4;
      }
      lastWasExpr = isExpr;
    }
    if (lastWasExpr) concatenationCost -= 3;
    if (typeof args[0] !== 'string' && typeof args[1] === 'string') {
      concatenationCost -= 3;
    }

    if (concatenationCost <= templateCost) return this.#es5Concatenation(args);

    return `\`${args
      .map((arg) => (typeof arg === 'string' ? arg : `\${${arg.expr}}`))
      .join('')}\``;
  }

  /**
   * @param args args (len >= 2)
   * @returns result expression
   */
  #es5Concatenation(args: ConcatenationArg[]): string {
    const str = args
      .map((arg) => (typeof arg === 'string' ? JSON.stringify(arg) : arg.expr))
      .join(' + ');

    // when the first two args are expression, we need to prepend "" + to force string
    // concatenation instead of number addition.
    return typeof args[0] !== 'string' && typeof args[1] !== 'string'
      ? `"" + ${str}`
      : str;
  }

  /**
   * @param expression expression
   * @param args arguments
   * @returns expression function code
   */
  expressionFunction(expression: string, args = ''): string {
    return this.supportsArrowFunction()
      ? `(${args}) => (${expression})`
      : `function(${args}) { ${expression}; }`;
  }

  /**
   * @returns empty function code
   */
  emptyFunction(): string {
    return this.supportsArrowFunction() ? 'x => {}' : 'function() {}';
  }

  /**
   * @param items items
   * @param value value
   * @returns destructure array code
   */
  destructureArray(items: string[], value: string): string {
    return this.supportsDestructuring()
      ? `var [${items.join(', ')}] = ${value};`
      : Template.asString(
          items.map((item, i) => `var ${item} = ${value}[${i}];`),
        );
  }

  /**
   * @param items items
   * @param value value
   * @returns destructure object code
   */
  destructureObject(items: string[], value: string): string {
    return this.supportsDestructuring()
      ? `var {${items.join(', ')}} = ${value};`
      : Template.asString(
          items.map(
            (item) => `var ${item} = ${value}${propertyAccess([item])};`,
          ),
        );
  }

  /**
   * @param args arguments
   * @param body body
   * @returns IIFE code
   */
  iife(args: string, body: string | string[]): string {
    return `(${this.basicFunction(args, body)})()`;
  }

  /**
   * @param variable variable
   * @param array array
   * @param body body
   * @returns for each code
   */
  forEach(variable: string, array: string, body: string | string[]): string {
    return this.supportsForOf()
      ? `for(const ${variable} of ${array}) {\n${Template.indent(body)}\n}`
      : `${array}.forEach(function(${variable}) {\n${Template.indent(body)}\n});`;
  }

  /**
   * @param options generation options
   * @returns generated error block
   */
  throwMissingModuleErrorBlock({ request }: { request?: string }): string {
    const err = `Cannot find module '${request}'`;
    return `var e = new Error(${JSON.stringify(err)}); e.code = 'MODULE_NOT_FOUND'; throw e;`;
  }

  /**
   * @param options generation options
   * @returns generated error function
   */
  throwMissingModuleErrorFunction({ request }: { request?: string }): string {
    return `function webpackMissingModule() { ${this.throwMissingModuleErrorBlock({ request })} }`;
  }

  /**
   * @param options generation options
   * @returns generated error IIFE
   */
  missingModule({ request }: { request?: string }): string {
    return `Object(${this.throwMissingModuleErrorFunction({ request })}())`;
  }

  /**
   * @param options generation options
   * @returns generated error statement
   */
  missingModuleStatement({ request }: { request?: string }): string {
    return `${this.missingModule({ request })};\n`;
  }

  /**
   * @param options generation options
   * @returns generated error code
   */
  missingModulePromise({ request }: { request?: string }): string {
    return `Promise.resolve().then(${this.throwMissingModuleErrorFunction({ request })})`;
  }
}
