import path from 'node:path';
import HtmlRspackPlugin from 'html-rspack-plugin';
export default [
  {
    plugins: [
      new HtmlRspackPlugin({
        filename: 'html-index.html',
        template:
          'html-loader!' + path.join(import.meta.dirname, 'template.html'),
      }),
    ],
  },
  {
    plugins: [
      new HtmlRspackPlugin({
        filename: 'pug-index.html',
        template:
          '@webdiscus/pug-loader!' +
          path.join(import.meta.dirname, 'template.pug'),
      }),
    ],
  },
];
