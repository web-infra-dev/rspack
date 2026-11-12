import { defineConfig } from '@rspack/cli';
import { type Configuration, container } from '@rspack/core';

const { ModuleFederationPlugin } = container;

const common: Configuration = {
  entry: {
    main: './index.js',
  },
  optimization: {
    runtimeChunk: 'single',
  },
};

const commonMF = {
  runtime: false as const,
  exposes: {
    './ComponentB': './ComponentB',
    './ComponentC': './ComponentC',
  },
  shared: ['@rspack/mocked-react'],
  shareScope: 'test-scope',
};

export default defineConfig([
  {
    ...common,
    output: {
      filename: '[name].js',
      uniqueName: 'mf-with-shareScope',
    },
    plugins: [
      new ModuleFederationPlugin({
        name: 'container2',
        library: { type: 'commonjs-module' },
        filename: 'container.js',
        remotes: {
          containerA: '../0-container-full/container.js',
          containerB: './container.js',
        },
        ...commonMF,
      }),
    ],
  },
  // {
  // 	...common,
  // 	experiments: {
  // 		// 	},
  // 	output: {
  // 		filename: "module/[name].mjs",
  // 		uniqueName: "mf-with-shareScope-mjs"
  // 	},
  // 	plugins: [
  // 		new ModuleFederationPlugin({
  // 			name: "container2",
  // 			library: { type: "module" },
  // 			filename: "module/container.mjs",
  // 			remotes: {
  // 				containerA: "../../../0-container-full/dist/module/container.mjs",
  // 				containerB: "./container.mjs"
  // 			},
  // 			...commonMF
  // 		})
  // 	],
  // 	target: "node14"
  // }
]);
