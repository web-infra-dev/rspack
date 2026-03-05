/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type binding from '@rspack/binding';

import type { Compilation } from './Compilation';
import type { StatsOptions, StatsValue } from './config';
import type { StatsCompilation } from './stats/statsFactoryUtils';

export type {
  StatsAsset,
  StatsChunk,
  StatsCompilation,
  StatsError,
  StatsModule,
} from './stats/statsFactoryUtils';

export class Stats {
  #inner: binding.JsStats;
  #compilation: Compilation;
  #innerMap: WeakMap<Compilation, binding.JsStats>;

  constructor(compilation: Compilation) {
    this.#inner = compilation.__internal_getInner().getStats();
    this.#compilation = compilation;
    this.#innerMap = new WeakMap([[this.compilation, this.#inner]]);
  }

  // use correct JsStats for child compilation
  #getInnerByCompilation(compilation: Compilation): binding.JsStats {
    if (this.#innerMap.has(compilation)) {
      return this.#innerMap.get(compilation)!;
    }
    const inner = compilation.__internal_getInner().getStats();
    this.#innerMap.set(compilation, inner);
    return inner;
  }

  get compilation() {
    if (this.#compilation.__internal__shutdown) {
      throw new Error(
        'Unable to access `Stats` after the compiler was shutdown',
      );
    }
    return this.#compilation;
  }

  get hash() {
    return this.compilation.hash;
  }

  get startTime() {
    return this.compilation.startTime;
  }

  get endTime() {
    return this.compilation.endTime;
  }

  hasErrors(): boolean {
    return (
      this.#compilation.errors.length > 0 ||
      this.#compilation.children.some((child) => child.getStats().hasErrors())
    );
  }

  hasWarnings(): boolean {
    const warnings = this.#compilation.hooks.processWarnings.call(
      this.#compilation.warnings,
    );
    return (
      warnings.length > 0 ||
      this.#compilation.children.some((child) => child.getStats().hasWarnings())
    );
  }

  toJson(opts?: StatsValue, forToString?: boolean): StatsCompilation {
    const options = this.compilation.createStatsOptions(opts, {
      forToString,
    });

    const statsFactory = this.compilation.createStatsFactory(options);

    const statsCompilationMap = new Map<
      Compilation,
      binding.JsStatsCompilation
    >();

    const stats = statsFactory.create('compilation', this.compilation, {
      compilation: this.compilation,
      getStatsCompilation: (
        compilation: Compilation,
      ): binding.JsStatsCompilation => {
        if (statsCompilationMap.has(compilation)) {
          return statsCompilationMap.get(compilation)!;
        }
        const innerStats = this.#getInnerByCompilation(compilation);
        options.warnings = false;
        const innerStatsCompilation = innerStats.toJson(options);
        statsCompilationMap.set(compilation, innerStatsCompilation);
        return innerStatsCompilation;
      },
      getInner: this.#getInnerByCompilation.bind(this),
    });
    return stats as StatsCompilation;
  }

  toString(opts?: StatsValue) {
    const options = this.compilation.createStatsOptions(opts, {
      forToString: true,
    });
    const statsFactory = this.compilation.createStatsFactory(options);

    const statsPrinter = this.compilation.createStatsPrinter(options);

    const statsCompilationMap = new Map<
      Compilation,
      binding.JsStatsCompilation
    >();

    const stats = statsFactory.create('compilation', this.compilation, {
      compilation: this.compilation,
      getStatsCompilation: (
        compilation: Compilation,
      ): binding.JsStatsCompilation => {
        if (statsCompilationMap.has(compilation)) {
          return statsCompilationMap.get(compilation)!;
        }
        const innerStats = this.#getInnerByCompilation(compilation);
        const innerStatsCompilation = innerStats.toJson(options);
        statsCompilationMap.set(compilation, innerStatsCompilation);
        return innerStatsCompilation;
      },
      getInner: this.#getInnerByCompilation.bind(this),
    });

    const result = statsPrinter.print('compilation', stats);

    return result === undefined ? '' : result;
  }
}

export function normalizeStatsPreset(options?: StatsValue): StatsOptions {
  if (typeof options === 'boolean' || typeof options === 'string')
    return presetToOptions(options);
  if (!options) return {};

  const obj = { ...presetToOptions(options.preset), ...options };
  delete obj.preset;
  return obj;
}

function presetToOptions(name?: boolean | string): StatsOptions {
  const preset = (typeof name === 'string' && name.toLowerCase()) || name;
  switch (preset) {
    case 'none':
      return {
        all: false,
      };
    case 'verbose':
      return {
        all: true,
        modulesSpace: Number.POSITIVE_INFINITY,
      };
    case 'errors-only':
      return {
        all: false,
        errors: true,
        errorsCount: true,
        logging: 'error',
        moduleTrace: true,
      };
    case 'errors-warnings':
      return {
        all: false,
        errors: true,
        errorsCount: true,
        warnings: true,
        warningsCount: true,
        logging: 'warn',
      };
    default:
      return {};
  }
}
