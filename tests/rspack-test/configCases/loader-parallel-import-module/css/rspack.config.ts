import { defineConfig, definePlugin } from '@rspack/cli';

export default defineConfig({
  output: {
    publicPath: '/public/',
  },
  entry: './index.js',
  module: {
    parser: {
      javascript: {
        // url: "relative"
      },
    },
    rules: [
      {
        dependency: 'url',
        issuer: /stylesheet\.js$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/[name][ext][query]',
        },
      },
      {
        oneOf: [
          {
            test: /other-stylesheet\.js$/,
            use: [
              {
                loader: './loader.mjs',
                options: {
                  publicPath: '/other/',
                  baseUri: 'my-schema://base',
                },
                parallel: true,
              },
            ],
            type: 'asset/source',
          },
          {
            test: /stylesheet\.js$/,
            use: [
              {
                loader: './loader.mjs',
                options: {
                  baseUri: 'my-schema://base',
                },
                parallel: true,
              },
            ],
            type: 'asset/source',
          },
        ],
      },
      {
        test: /\.jpg$/,
        type: 'asset/resource',
        generator: {
          filename: 'assets/[name][ext]',
        },
      },
    ],
  },
  plugins: [
    definePlugin((compiler) =>
      compiler.hooks.done.tap('test case', (stats) => {
        try {
          expect(stats.compilation.getAsset('assets/file.png')).toHaveProperty(
            'info',
            expect.objectContaining({ sourceFilename: 'file.png' }),
          );
          expect(stats.compilation.getAsset('assets/file.jpg')).toHaveProperty(
            'info',
            expect.objectContaining({ sourceFilename: 'file.jpg' }),
          );
          const auxiliaryFiles =
            stats.compilation.namedChunks.get('main')?.auxiliaryFiles;
          expect(auxiliaryFiles).toContain('assets/file.png');
        } catch (e) {
          console.log(stats.toString({ colors: true, orphanModules: true }));
          throw e;
        }
      }),
    ),
  ],
});
