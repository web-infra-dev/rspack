import type { StatsFactory } from './StatsFactory';

type ContextFreeItems = {
  name: string;
  getName: (...args: any[]) => any;
  extractors: ((...args: any[]) => any)[];
  canUse: () => boolean;
};

export const factoriesCreatingNativeStats = new WeakSet<StatsFactory>();
export const contextFreeStatsItems = new WeakMap<
  StatsFactory,
  Map<string, ContextFreeItems>
>();
