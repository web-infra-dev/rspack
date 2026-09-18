import {
  type BuiltinPlugin,
  BuiltinPluginName,
  type RawExternalItemFnCtx,
  type RawExternalsPluginOptions,
} from '@rspack/binding';

import type {
  ExternalItem,
  ExternalItemFunctionData,
  ExternalItemValue,
  Externals,
} from '..';
import { getRawResolve } from '../config/adapter';
import type { ResolveCallback } from '../config/adapterRuleUse';
import type { ResolveRequest } from '../Resolver';
import { createBuiltinPlugin, RspackBuiltinPlugin } from './base';

export class ExternalsPlugin extends RspackBuiltinPlugin {
  name = BuiltinPluginName.ExternalsPlugin;

  #resolveRequestCache = new Map<string, ResolveRequest>();

  constructor(
    private type: string,
    private externals: Externals,
    private placeInInitial?: boolean,
    private fallbackType?: string,
  ) {
    super();
  }

  raw(): BuiltinPlugin | undefined {
    const type = this.type;
    const externals = this.externals;
    const raw: RawExternalsPluginOptions = {
      type,
      fallbackType: this.fallbackType,
      externals: (Array.isArray(externals) ? externals : [externals])
        .filter(Boolean)
        .map((item) => this.#getRawExternalItem(item)),
      placeInInitial: this.placeInInitial ?? false,
    };
    return createBuiltinPlugin(this.name, raw);
  }

  #processResolveResult = (
    text: string | undefined,
  ): ResolveRequest | undefined => {
    if (!text) return undefined;

    let resolveRequest = this.#resolveRequestCache.get(text);
    if (!resolveRequest) {
      resolveRequest = JSON.parse(text) as ResolveRequest;
      this.#resolveRequestCache.set(text, resolveRequest);
    }
    return Object.assign({}, resolveRequest);
  };

  // Reference: webpack/enhanced-resolve#255
  // Handle fragment escaping in resolve results:
  // - `#` can be escaped as `\0#` to prevent fragment parsing
  // - enhanced-resolve resolves `#` ambiguously as both path and fragment
  // - Example: `./some#thing` could resolve to `.../some.js#thing` or `.../some#thing.js`
  // - When `#` is part of the path, it gets escaped as `\0#` in the result
  // - We replace `\0#` with zero-width space + `#` (\u200b#) for compatibility
  #processRequest(req: ResolveRequest): string {
    return `${req.path.replace(/#/g, '\u200b#')}${req.query.replace(/#/g, '\u200b#')}${req.fragment}`;
  }

  #getRawExternalItem = (item: ExternalItem | undefined): RawExternalItem => {
    if (typeof item === 'string' || item instanceof RegExp) {
      return item;
    }

    if (typeof item === 'function') {
      const processResolveResult = this.#processResolveResult;
      // Whether the native binding exposes the per-field getters; resolved
      // once per externals item so the check stays out of the per-call path.
      let lazyCtx: boolean | undefined;

      return async (ctx: RawExternalItemFnCtx) => {
        return new Promise((resolve, reject) => {
          // Track which inputs the user function actually reads so the native
          // side only caches on the fields that can affect the result. Bit 32
          // marks a call that used `getResolve` as uncacheable.
          let observed = 0;
          let resolveUsed = false;
          // Read fields lazily through the native ctx so unused inputs are
          // never materialized.
          const rawCtx = ctx as any;
          // Older native bindings only expose the bulk `data()` accessor. The
          // check must not read a field, otherwise every call would materialize
          // it just to detect the binding.
          const lazy = (lazyCtx ??= 'request' in rawCtx);
          const data = lazy ? undefined : ctx.data();
          // Fields the callback assigned keep the writable behaviour they had
          // while they were plain data properties; overrides stay per call.
          let assigned: any;
          const contextInfo = {
            get issuer() {
              observed |= 8;
              if (assigned && 'issuer' in assigned) return assigned.issuer;
              return lazy ? rawCtx.issuer : data!.contextInfo.issuer;
            },
            set issuer(value: string) {
              (assigned ??= {}).issuer = value;
            },
            get issuerLayer() {
              observed |= 16;
              if (assigned && 'issuerLayer' in assigned) {
                return assigned.issuerLayer;
              }
              return (
                (lazy ? rawCtx.issuerLayer : data!.contextInfo.issuerLayer) ??
                null
              );
            },
            set issuerLayer(value: string | null) {
              (assigned ??= {}).issuerLayer = value;
            },
          };
          const rawResult = (
            result: ExternalItemValue | undefined,
            externalType: any,
          ) => ({
            result: getRawExternalItemValueFormFnResult(result),
            externalType,
            observed: (resolveUsed ? 32 : 0) | observed,
          });
          const promise = item(
            {
              get request() {
                observed |= 1;
                if (assigned && 'request' in assigned) return assigned.request;
                return lazy ? rawCtx.request : data!.request;
              },
              set request(value: string) {
                (assigned ??= {}).request = value;
              },
              get dependencyType() {
                observed |= 4;
                if (assigned && 'dependencyType' in assigned) {
                  return assigned.dependencyType;
                }
                return lazy ? rawCtx.dependencyType : data!.dependencyType;
              },
              set dependencyType(value: string) {
                (assigned ??= {}).dependencyType = value;
              },
              get context() {
                observed |= 2;
                if (assigned && 'context' in assigned) return assigned.context;
                return lazy ? rawCtx.context : data!.context;
              },
              set context(value: string) {
                (assigned ??= {}).context = value;
              },
              get contextInfo() {
                observed |= 24;
                if (assigned && 'contextInfo' in assigned) {
                  return assigned.contextInfo;
                }
                return contextInfo;
              },
              set contextInfo(value: ExternalItemFunctionData['contextInfo']) {
                (assigned ??= {}).contextInfo = value;
              },
              getResolve: (options) => {
                resolveUsed = true;
                const rawResolve = options ? getRawResolve(options) : undefined;
                const resolve = ctx.getResolve(rawResolve);

                return (
                  context: string,
                  request: string,
                  callback?: ResolveCallback,
                ) => {
                  if (callback) {
                    resolve(context, request, (error, text) => {
                      if (error) {
                        callback(error);
                      } else {
                        const req = processResolveResult(text);
                        callback(
                          null,
                          req ? this.#processRequest(req) : false,
                          req,
                        );
                      }
                    });
                  } else {
                    return new Promise((promiseResolve, promiseReject) => {
                      resolve(context, request, (error, text) => {
                        if (error) {
                          promiseReject(error);
                        } else {
                          const req = processResolveResult(text);
                          promiseResolve(
                            req ? this.#processRequest(req) : undefined,
                          );
                        }
                      });
                    });
                  }
                };
              },
            },
            (err, result, type) => {
              if (err) reject(err);
              resolve(rawResult(result, type));
            },
          ) as Promise<ExternalItemValue> | ExternalItemValue | undefined;
          if ((promise as Promise<ExternalItemValue>)?.then) {
            (promise as Promise<ExternalItemValue>).then(
              (result) => resolve(rawResult(result, undefined)),
              (e) => reject(e),
            );
          } else if (item.length === 1) {
            // No callback and no promise returned, regarded as a synchronous function
            resolve(
              rawResult(promise as ExternalItemValue | undefined, undefined),
            );
          }
        });
      };
    }
    if (typeof item === 'object') {
      return Object.fromEntries(
        Object.entries(item).map(([k, v]) => [k, getRawExternalItemValue(v)]),
      );
    }
    throw new TypeError(`Unexpected type of external item: ${typeof item}`);
  };
}

type ArrayType<T> = T extends (infer R)[] ? R : never;
type RecordValue<T> = T extends Record<any, infer R> ? R : never;
type RawExternalItem = ArrayType<RawExternalsPluginOptions['externals']>;
type RawExternalItemValue = RecordValue<RawExternalItem>;

function getRawExternalItemValueFormFnResult(result?: ExternalItemValue) {
  return result === undefined ? result : getRawExternalItemValue(result);
}

function getRawExternalItemValue(
  value: ExternalItemValue,
): RawExternalItemValue {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return Object.fromEntries(
      Object.entries(value).map(([k, v]) => [k, Array.isArray(v) ? v : [v]]),
    );
  }
  return value;
}
