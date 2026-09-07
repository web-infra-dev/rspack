globalThis.__sharedRuns = (globalThis.__sharedRuns || 0) + 1;

export const value = globalThis.__sharedRuns;
