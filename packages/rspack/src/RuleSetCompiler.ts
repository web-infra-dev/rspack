import type { RuleSetLoaderWithOptions } from './config/types';

export type LoaderParallel = Exclude<
  RuleSetLoaderWithOptions['parallel'],
  undefined | false
>;

export class ParallelLoaderOptions {
  constructor(
    public readonly options: unknown,
    public readonly parallel: LoaderParallel,
  ) {}
}

class RuleSetCompiler {
  references: Map<string, any>;
  constructor() {
    this.references = new Map();
  }
}

export { RuleSetCompiler };
