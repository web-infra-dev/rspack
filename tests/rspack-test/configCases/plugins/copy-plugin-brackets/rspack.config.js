const { CopyRspackPlugin } = require('@rspack/core');

module.exports = {
  entry: './index.js',
  target: 'node',
  plugins: [
    new CopyRspackPlugin({
      patterns: [
        {
          from: 'src/directory[1]',
          to: 'from-directory',
        },
        'src/file[1].txt',
        {
          from: 'src/file[1].txt',
          to: 'from-object/file[1].txt',
        },
        {
          from: 'src/dotfiles/**/.ENV',
          to: 'from-dotfile',
          globOptions: {
            caseSensitiveMatch: false,
          },
        },
      ],
    }),
  ],
  output: {
    clean: true,
  },
};
