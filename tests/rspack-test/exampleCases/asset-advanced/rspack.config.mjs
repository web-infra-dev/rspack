import svgToMiniDataURI from 'mini-svg-data-uri';
export default {
  output: {
    assetModuleFilename: 'images/[hash][ext]',
  },
  module: {
    rules: [
      {
        test: /\.(png|jpg)$/,
        type: 'asset',
      },
      {
        test: /\.svg$/,
        type: 'asset',
        generator: {
          dataUrl: (content) => {
            if (typeof content !== 'string') {
              content = content.toString();
            }

            return svgToMiniDataURI(content);
          },
        },
      },
    ],
  },
};
