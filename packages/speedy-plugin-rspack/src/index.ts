import { SpeedyPlugin } from '@speedy-js/speedy-types';
import { RspackBundler } from './rspack-bundler';

export function speedyPluginRspack(): SpeedyPlugin {
  return {
    name: 'rspack',
    apply(compiler) {
      compiler.hooks.compilation.tap('rspack', async () => {
        compiler.compilation = new RspackBundler(compiler);
        await compiler.compilation.build();
        await compiler.hooks.done.promise();
        return 'rspack';
      });
    },
  };
}
