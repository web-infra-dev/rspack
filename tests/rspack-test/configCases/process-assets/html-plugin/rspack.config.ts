import { defineConfig } from '@rspack/cli';
import type { Compiler } from '@rspack/core';
import { rspack } from '@rspack/core';

const {
  sources: { RawSource, OriginalSource, ReplaceSource },
  Compilation,
  util: { createHash },
  optimize: { RealContentHashPlugin },
} = rspack;

class VerifyAdditionalAssetsPlugin {
  stage: number;

  constructor(stage: number) {
    this.stage = stage;
  }

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap(
      'VerifyAdditionalAssetsPlugin',
      (compilation) => {
        const alreadySeenAssets = new Set();
        compilation.hooks.processAssets.tap(
          {
            name: 'VerifyAdditionalAssetsPlugin',
            stage: this.stage,
            // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
            additionalAssets: true,
          },
          (assets) => {
            for (const asset of Object.keys(assets)) {
              expect(alreadySeenAssets).not.toContain(asset);
              alreadySeenAssets.add(asset);
            }
          },
        );
      },
    );
  }
}

class HtmlPlugin {
  entrypoints: string[];

  constructor(entrypoints: string[]) {
    this.entrypoints = entrypoints;
  }

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('html-plugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'html-plugin',
          stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL,
        },
        () => {
          const publicPath = compilation.outputOptions.publicPath;
          const files = [];
          for (const name of this.entrypoints) {
            for (const file of compilation.entrypoints.get(name)!.getFiles())
              files.push(file);
          }
          const toScriptTag = (file: string) => {
            const asset = compilation.getAsset(file)!;
            const hash = createHash('sha512');
            hash.update(Buffer.from(asset.source.source()));
            const integrity = `sha512-${hash.digest('base64')}`;
            compilation.updateAsset(
              file,
              (x) => x,
              (assetInfo) => ({
                ...assetInfo,
                contenthash: Array.isArray(assetInfo.contenthash)
                  ? [...new Set([...assetInfo.contenthash, integrity])]
                  : assetInfo.contenthash
                    ? [assetInfo.contenthash, integrity]
                    : integrity,
              }),
            );
            return `<script src="${
              publicPath === 'auto' ? '' : publicPath
            }${file}" integrity="${integrity}"></script>`;
          };
          compilation.emitAsset(
            'index.html',
            new OriginalSource(
              `<html>
	<body>
${files.map((file) => `		${toScriptTag(file)}`).join('\n')}
	</body>
</html>`,
              'index.html',
            ),
          );
        },
      );
    });
  }
}

class HtmlInlinePlugin {
  inline: RegExp;

  constructor(inline: RegExp) {
    this.inline = inline;
  }

  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('html-inline-plugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'html-inline-plugin',
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE,
          // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
          additionalAssets: true,
        },
        (assets) => {
          const publicPath = compilation.outputOptions.publicPath;
          for (const name of Object.keys(assets)) {
            if (/\.html$/.test(name)) {
              const asset = compilation.getAsset(name)!;
              const content = asset.source.source().toString();
              const matches = [];
              const regExp =
                /<script\s+src\s*=\s*"([^"]+)"(?:\s+[^"=\s]+(?:\s*=\s*(?:"[^"]*"|[^\s]+))?)*\s*><\/script>/g;
              let match = regExp.exec(content);
              while (match) {
                let url = match[1];
                if (
                  typeof publicPath === 'string' &&
                  url.startsWith(publicPath)
                )
                  url = url.slice(publicPath.length);
                if (this.inline.test(url)) {
                  const asset = compilation.getAsset(url)!;
                  matches.push({
                    start: match.index,
                    length: match[0].length,
                    asset,
                  });
                }
                match = regExp.exec(content);
              }
              if (matches.length > 0) {
                const newSource = new ReplaceSource(asset.source, name);
                for (const { start, length, asset } of matches) {
                  newSource.replace(
                    start,
                    start + length - 1,
                    `<script>${asset.source.source()}</script>`,
                  );
                }
                compilation.updateAsset(name, newSource);
              }
            }
          }
        },
      );
    });
  }
}

class SriHashSupportPlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('sri-hash-support-plugin', (compilation) => {
      RealContentHashPlugin.getCompilationHooks(compilation).updateHash.tap(
        'sri-hash-support-plugin',
        (input, oldHash) => {
          expect(Array.isArray(input)).toBe(true);
          expect(input.every(Buffer.isBuffer)).toBe(true);
          expect(typeof oldHash).toBe('string');
          if (/^sha512-.{88}$/.test(oldHash) && input.length === 1) {
            const hash = createHash('sha512');
            hash.update(input[0]);
            return `sha512-${hash.digest('base64')}`;
          }
        },
      );
    });
  }
}

class HtmlMinimizePlugin {
  apply(compiler: Compiler) {
    compiler.hooks.compilation.tap('html-minimize-plugin', (compilation) => {
      compilation.hooks.processAssets.tap(
        {
          name: 'html-minimize-plugin',
          stage: Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE,
          // @ts-expect-error Rspack does not expose the additionalAssets tap option yet.
          additionalAssets: true,
        },
        (assets) => {
          for (const name of Object.keys(assets)) {
            if (/\.html$/.test(name)) {
              compilation.updateAsset(
                name,
                (source) =>
                  new RawSource(
                    source.source().toString().replace(/\s+/g, ' '),
                  ),
                (assetInfo) => ({
                  ...assetInfo,
                  minimized: true,
                }),
              );
            }
          }
        },
      );
    });
  }
}

export default defineConfig({
  mode: 'production',
  entry: {
    test: { import: './index.js', filename: 'test.js' },
    inline: './inline.js',
    normal: './normal.js',
  },
  output: {
    filename: '[name]-[contenthash].js',
  },
  optimization: {
    minimize: true,
    minimizer: ['...', new HtmlMinimizePlugin()],
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  plugins: [
    new VerifyAdditionalAssetsPlugin(
      Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL - 1,
    ),
    // new VerifyAdditionalAssetsPlugin(Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE),
    // new VerifyAdditionalAssetsPlugin(Compilation.PROCESS_ASSETS_STAGE_REPORT),
    new HtmlPlugin(['inline', 'normal']),
    new HtmlInlinePlugin(/inline/),
    new SriHashSupportPlugin(),
  ],
});
