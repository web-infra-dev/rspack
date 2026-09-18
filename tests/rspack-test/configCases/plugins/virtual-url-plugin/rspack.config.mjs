import fs from 'node:fs';
import path from 'node:path';
import { rspack } from '@rspack/core';
import { fileURLToPath } from 'node:url';

const { VirtualUrlPlugin } = rspack.experiments.schemes;

const watchDir = path.join(import.meta.dirname, './routes');

/** @type {import('webpack').Configuration} */
const config = {
  plugins: [
    new VirtualUrlPlugin({
      routes() {
        const files = fs.readdirSync(watchDir);
        return `
					export const routes = {
						${files.map((key) => `${key.split('.')[0]}: () => import('./routes/${key}')`).join(',\n')}
					}
				`;
      },
      app: "export const app = 'app'",
      config: {
        type: '.json',
        source() {
          return '{"name": "virtual-url-plugin"}';
        },
      },
      ts: {
        type: '.ts',
        source() {
          return `interface Info {
						name: string;
					}
					export const ts = 'const';`;
        },
      },
      style: {
        type: '.css',
        source() {
          return 'body{background-color: powderblue;}';
        },
      },
      txt: {
        type: '.txt',
        source() {
          return 'Hello world';
        },
      },
    }),
  ],
  module: {
    rules: [
      {
        test: /\.ts/,
        use: [
          {
            loader: fileURLToPath(
              import.meta.resolve('./strip-types-loader.mjs'),
            ),
          },
          {
            loader: fileURLToPath(import.meta.resolve('./babel-loader.mjs')),
          },
        ],
      },
      {
        test: /\.txt/,
        type: 'asset/source',
      },
      {
        test: /\.css$/,
        type: 'css/auto',
      },
    ],
  },
};

export default config;
