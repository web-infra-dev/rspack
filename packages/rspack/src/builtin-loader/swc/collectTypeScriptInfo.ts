export type CollectTypeScriptInfoOptions = {
  /**
   * Whether to collect type exports information for `typeReexportsPresence`.
   * This is used to check type exports of submodules when running in `'tolerant'` mode.
   * @default false
   */
  typeExports?: boolean;
  /**
   * Whether to collect information about exported `enum`s.
   * - `true` will collect all `enum` information, including `const enum`s and regular `enum`s.
   * - `false` will not collect any `enum` information.
   * - `'const-only'` will gather only `const enum`s, enabling Rspack to perform cross-module
   * inlining optimizations for them.
   * @default false
   */
  exportedEnum?: boolean | 'const-only';
};

export function resolveCollectTypeScriptInfo(
  options: CollectTypeScriptInfoOptions,
) {
  return {
    typeExports: options.typeExports,
    exportedEnum:
      options.exportedEnum === true
        ? 'all'
        : options.exportedEnum === false || options.exportedEnum === undefined
          ? 'none'
          : 'const-only',
  };
}
