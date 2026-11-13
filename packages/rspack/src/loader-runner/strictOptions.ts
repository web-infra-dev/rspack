import { createRequire } from 'node:module';
import {
  getSerializedLoaderOptions,
  registerStrictLoader as bindingRegisterStrictLoader,
} from '@rspack/binding';
import type { Compiler } from '../Compiler';
import type { RuleSetLoaderWithOptions } from '../config/types';
import { prepareStrictLoaderPool } from './service';
import { strictLoaderError } from './utils';

const require = createRequire(import.meta.url);
type LoaderParallel = RuleSetLoaderWithOptions['parallel'];

type SerializedLoaderReference = { options: unknown; parallel: LoaderParallel };
type StrictCompilerOptions = {
  context: Compiler['context'];
  options: Pick<Compiler['options'], 'mode' | 'devtool' | 'loader'>;
  hashFunction: Compiler['options']['output']['hashFunction'];
};
export type StrictLoaderCompiler = StrictCompilerOptions & {
  root: object;
  __internal__ruleSet: {
    references: { get(ident: string): unknown };
  };
};

function serializeLoaderValue(value: unknown): Buffer | undefined {
  if (IS_BROWSER || process.env.WASM) return undefined;
  const { serialize } = require('node:v8') as typeof import('node:v8');
  try {
    return serialize(value);
  } catch {
    return undefined;
  }
}

function deserializeLoaderValue(bytes: Buffer): unknown {
  const { deserialize } = require('node:v8') as typeof import('node:v8');
  return deserialize(bytes);
}

function getLoaderReference(
  handle: number,
  ident: string,
): SerializedLoaderReference | undefined {
  const bytes = getSerializedLoaderOptions(handle, ident);
  return bytes
    ? (deserializeLoaderValue(bytes) as SerializedLoaderReference)
    : undefined;
}

const PARALLEL_SUFFIX = '$$parallelism';

export function createStrictLoaderCompiler(
  handle: number,
): StrictLoaderCompiler {
  const bytes = getSerializedLoaderOptions(handle);
  if (!bytes) throw strictLoaderError('Compiler options');
  const compiler = deserializeLoaderValue(bytes) as StrictCompilerOptions;

  const references = new Map<string, SerializedLoaderReference | undefined>();
  return {
    ...compiler,
    root: compiler,
    __internal__ruleSet: {
      references: {
        get(ident) {
          const isParallel = ident.endsWith(PARALLEL_SUFFIX);
          const optionsIdent = isParallel
            ? ident.slice(0, -PARALLEL_SUFFIX.length)
            : ident;
          if (!references.has(optionsIdent)) {
            references.set(
              optionsIdent,
              getLoaderReference(handle, optionsIdent),
            );
          }
          const value = references.get(optionsIdent);
          if (isParallel) return value?.parallel;
          if (!value)
            throw strictLoaderError(
              `Non-serializable loader options for '${ident}'`,
            );
          return value.options;
        },
      },
    },
  };
}

const compilerHandles = new WeakMap<Compiler, number>();
const strictCompilers = new WeakSet<Compiler>();
let nextCompilerHandle = 0;

export function getCompilerHandle(compiler: Compiler): number {
  let owner = compilerHandles.get(compiler);
  if (owner === undefined) {
    owner = ++nextCompilerHandle;
    compilerHandles.set(compiler, owner);
  }
  return owner;
}

export function registerStrictLoader(
  compiler: Compiler,
  ident: string,
  options: unknown,
): void {
  const serialized = serializeLoaderValue({ options, parallel: 'strict' });
  if (!serialized) {
    throw strictLoaderError(`Non-serializable loader options for '${ident}'`);
  }
  let compilerOptions: Buffer | undefined;
  if (!strictCompilers.has(compiler)) {
    const { context, options } = compiler;
    const { mode, devtool, loader, output } = options;
    compilerOptions = serializeLoaderValue({
      context,
      options: { mode, devtool, loader },
      hashFunction: output.hashFunction,
    } satisfies StrictCompilerOptions);
    if (!compilerOptions)
      throw strictLoaderError('Non-serializable compiler loader options');
  }
  bindingRegisterStrictLoader(
    getCompilerHandle(compiler),
    ident,
    serialized,
    compilerOptions,
  );
  if (compilerOptions) strictCompilers.add(compiler);
  prepareStrictLoaderPool();
}
