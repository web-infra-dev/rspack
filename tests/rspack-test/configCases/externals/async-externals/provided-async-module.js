globalThis.__rspackProvidedAsyncSideEffect = false;

await Promise.resolve();

globalThis.__rspackProvidedAsyncSideEffect = true;

export const inlined = 42;
