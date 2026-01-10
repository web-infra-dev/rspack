export type TransformImportConfig = {
  source: string;
  namedImport?: boolean;
  output: string[];
  exclude?: string[];
};

export type TransformImportOptions = TransformImportConfig[];

type RawTransformImportConfig = {
  source: string;
  namedImport?: boolean;
  output: string[];
  exclude?: string[];
};

function resolveTransformImport(
  transformImport: TransformImportOptions,
): RawTransformImportConfig[] | undefined {
  if (!transformImport) {
    return undefined;
  }

  return transformImport.map((config) => ({
    source: config.source,
    namedImport: config.namedImport ?? true,
    output: config.output,
    exclude: config.exclude,
  }));
}

export { resolveTransformImport };
