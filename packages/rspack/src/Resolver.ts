import type binding from '@rspack/binding';
import type { ResolveCallback } from './config/adapterRuleUse';

export type ResolveContext = {
  contextDependencies?: {
    add: (context: string) => void;
  };
  missingDependencies?: {
    add: (dependency: string) => void;
  };
  fileDependencies?: {
    add: (dependency: string) => void;
  };
};

export type ResourceData = binding.JsResourceData;

type JsonValueTypes =
  null | string | number | boolean | JsonObjectTypes | JsonValueTypes[];

type JsonObjectTypes = { [index: string]: JsonValueTypes } & {
  [index: string]:
    | undefined
    | null
    | string
    | number
    | boolean
    | JsonObjectTypes
    | JsonValueTypes[];
};

export interface ResolveRequest {
  path: string;
  query: string;
  fragment: string;
  /** Parsed contents of the package description file, when available. */
  descriptionFileData?: JsonObjectTypes | null;
  descriptionFilePath?: string;
  fileDependencies?: string[];
  missingDependencies?: string[];
  contextDependencies?: string[];
}

interface ResolveResponse {
  error?: string;
  request?: ResolveRequest;
  fileDependencies?: string[];
  missingDependencies?: string[];
}

export class Resolver {
  #binding: binding.JsResolver;

  constructor(binding: binding.JsResolver) {
    this.#binding = binding;
  }

  resolveSync(_context: object, path: string, request: string): string | false {
    return this.#binding.resolveSync(path, request) ?? false;
  }

  resolve(
    _context: object,
    path: string,
    request: string,
    resolveContext: ResolveContext,
    callback: ResolveCallback,
  ): void {
    this.#binding.resolve(path, request, (error, text) => {
      if (error) {
        callback(error);
        return;
      }
      const res = text ? (JSON.parse(text) as ResolveResponse) : undefined;

      if (res?.fileDependencies) {
        for (const file of res.fileDependencies) {
          resolveContext?.fileDependencies?.add(file);
        }
      }

      if (res?.missingDependencies) {
        for (const missing of res.missingDependencies) {
          resolveContext?.missingDependencies?.add(missing);
        }
      }

      if (res?.error !== undefined) {
        callback(new Error(res.error));
        return;
      }

      const req = res?.request;
      if (req) {
        req.fileDependencies = res?.fileDependencies ?? [];
        req.missingDependencies = res?.missingDependencies ?? [];
      }

      callback(
        null,
        req
          ? `${req.path.replace(/#/g, '\u200b#')}${req.query.replace(/#/g, '\u200b#')}${req.fragment}`
          : false,
        req,
      );
    });
  }
}
