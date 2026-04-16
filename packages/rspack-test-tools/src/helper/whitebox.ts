const CASE_FILTERS = (process.env.RSPACK_WASM_WHITEBOX_CASE_FILTER || '')
  .split('|')
  .map((filter) => filter.trim())
  .filter(Boolean);

const FILE_FILTERS = (process.env.RSPACK_WASM_WHITEBOX_FILE_FILTER || '')
  .split('|')
  .map((filter) => filter.trim())
  .filter(Boolean);

function matchFilter(value: string, filters: string[]) {
  return (
    filters.length === 0 || filters.some((filter) => value.includes(filter))
  );
}

export function isWasmWhiteboxEnabled() {
  return !!(
    process.env.CI &&
    process.env.WASM &&
    process.env.RSPACK_WASM_WHITEBOX_DEBUG === '1'
  );
}

export function shouldLogWhiteboxCase(name: string) {
  return isWasmWhiteboxEnabled() && matchFilter(name, CASE_FILTERS);
}

export function shouldLogWhiteboxFile(filename: string) {
  return isWasmWhiteboxEnabled() && matchFilter(filename, FILE_FILTERS);
}

export function formatWhiteboxError(error: unknown) {
  if (error instanceof Error) {
    return error.stack || error.message;
  }
  return String(error);
}

export function whiteboxLogCase(
  name: string,
  stage: string,
  details?: Record<string, unknown>,
) {
  if (!shouldLogWhiteboxCase(name)) {
    return;
  }

  const suffix =
    details && Object.keys(details).length > 0
      ? ` ${JSON.stringify(details)}`
      : '';
  console.error(`[wasm-whitebox] ${name} :: ${stage}${suffix}`);
}
