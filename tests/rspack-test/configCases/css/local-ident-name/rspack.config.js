const common = {
  externals: {
    path: 'node-commonjs path',
  },
  mode: 'development',
  devtool: false,
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'css/auto',
        use: ['less-loader'],
        generator: {
          localIdentName: '[path][name][ext]__[local]',
        },
      },
      {
        test: /\.css$/,
        type: 'css/auto',
        oneOf: [
          {
            test: /fast-path\.module\.css$/,
            generator: {
              localIdentHashDigestLength: 8,
              localIdentName: '[hash]-[local]',
            },
          },
          {
            resourceQuery: /\?hash$/,
            generator: {
              localIdentHashDigest: 'hex',
              localIdentHashDigestLength: 20,
              localIdentHashFunction: 'md4',
              localIdentName: '[hash]',
            },
          },
          {
            resourceQuery: /\?hash-local$/,
            generator: {
              localIdentHashDigest: 'hex',
              localIdentHashDigestLength: 20,
              localIdentHashFunction: 'md4',
              localIdentName: '[hash]-[local]',
            },
          },
          {
            resourceQuery: /\?fullhash-local$/,
            generator: {
              localIdentHashDigest: 'base64url',
              localIdentHashDigestLength: 6,
              localIdentHashFunction: 'md4',
              localIdentName: '[fullhash]-[local]',
            },
          },
          {
            resourceQuery: /\?path-name-local$/,
            generator: {
              localIdentName: '[path][name]__[local]',
            },
          },
          {
            resourceQuery: /\?file-local$/,
            generator: {
              localIdentName: '[file]__[local]',
            },
          },
          {
            resourceQuery: /\?q$/,
            resourceFragment: /#f$/,
            generator: {
              localIdentName: '[file][query][fragment]__[local]',
            },
          },
          {
            resourceQuery: /\?uniqueName-id-contenthash$/,
            generator: {
              localIdentName: '[uniqueName]-[id]-[contenthash]',
            },
          },
          {
            resourceQuery: /\?hash-local-custom$/,
            generator: {
              localIdentHashDigest: 'hex',
              localIdentHashDigestLength: 20,
              localIdentHashFunction: 'md4',
              localIdentName: '[hash]-[local]',
            },
          },
        ],
      },
    ],
  },
};

/** @type {import("@rspack/core").Configuration} */
module.exports = [
  {
    ...common,
    target: 'web',
  },
  {
    ...common,
    target: 'node',
  },
];
