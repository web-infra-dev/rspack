globalThis.__CJS_DYNAMIC_KEY_READ_COUNT__ = 0;
globalThis.__CJS_IMPURE_RHS_COUNT__ = 0;
globalThis.__CJS_SHADOWED_WRITE_COUNT__ = 0;

globalThis.__getCjsDynamicExportKey = () => {
  globalThis.__CJS_DYNAMIC_KEY_READ_COUNT__ += 1;
  return 'dynamic';
};

globalThis.__recordCjsImpureRhs = () => {
  globalThis.__CJS_IMPURE_RHS_COUNT__ += 1;
  return 1;
};
