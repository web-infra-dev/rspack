import type { Compiler } from '@rspack/core';

export default class TestPlugin {
  declare cb: (compiler: Compiler, list: string[]) => void;

  constructor(cb: TestPlugin['cb']) {
    this.cb = cb;
  }
  apply(compiler: Compiler) {
    const list: string[] = [];
    this.cb(compiler, list);
    compiler.hooks.compilation.tap(TestPlugin.name, (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: TestPlugin.name,
          // ProcessAssetsStageAdditions
          stage: -100,
        },
        (assets) => {
          for (const file of Object.keys(assets)) {
            compilation.updateAsset(file, (old) => {
              const newContent = `${list.join('\n')}\n${old.source().toString()}`;
              return new compiler.rspack.sources.RawSource(newContent);
            });
          }
        },
      );
    });
  }
}
