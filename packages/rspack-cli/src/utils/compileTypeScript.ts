import { rspack } from '@rspack/core';

type ModuleType = 'commonjs' | 'es6';

const injectInlineSourceMap = ({
  code,
  map,
}: {
  code: string;
  map: string | undefined;
}): string => {
  if (map) {
    const base64Map = Buffer.from(map, 'utf8').toString('base64');
    const sourceMapContent = `//# sourceMappingURL=data:application/json;charset=utf-8;base64,${base64Map}`;
    return `${code}\n${sourceMapContent}`;
  }
  return code;
};

export const compileTypeScript = (
  sourcecode: string,
  filename: string,
  moduleType: ModuleType,
) => {
  const { code, map } = rspack.experiments.swc.transformSync(sourcecode, {
    jsc: {
      parser: {
        syntax: 'typescript',
        tsx: false,
        decorators: true,
        dynamicImport: true,
      },
    },
    filename,
    module: { type: moduleType },
    sourceMaps: true,
    isModule: true,
  });

  return injectInlineSourceMap({ code, map });
};
