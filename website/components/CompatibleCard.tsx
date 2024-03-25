import './CompatibleCard.scss';
import * as i18n from './i18n';
enum compatibleStats {
  COMPATIBLE,
  INCOMPATIBLE,
}
interface CardMeta {
  name: string;
  url: string;
  status: string;
  description?: string;
  remark?: string;
}

const CompatibleCardItem = ({
  name,
  url,
  status,
  description,
  remark,
}: CardMeta) => {
  return (
    <div className="component-card">
      <div className="component-card-title-line">
        <a className="component-card-link" href={url}>
          {name}
        </a>
        <div className="component-card-space"></div>
        <div className="component-card-status">{status}</div>
      </div>
      {description && <div className="component-card-desc">{description}</div>}
      {remark && <div>{remark}</div>}
    </div>
  );
};

export const PluginCompatibleCardList = ({ lang }: { lang: 'zh' | 'en' }) => {
  const pluginList: CardMeta[] = [
    {
      name: 'webpack.DefinePlugin',
      url: 'https://webpack.js.org/plugins/define-plugin/',
      status: i18n[lang]['compatible'],
      description: i18n[lang]['define-plugin-desc'],
    },
    {
      name: 'webpack.BannerPlugin',
      url: 'https://webpack.js.org/plugins/banner-plugin',
      status: i18n[lang]['compatible'],
      description: i18n[lang]['banner-plugin-desc'],
    },
    {
      name: 'webpack.HotModuleReplacementPlugin',
      url: 'https://webpack.js.org/plugins/hot-module-replacement-plugin',
      status: i18n[lang]['included'],
    },
    {
      name: 'html-webpack-plugin',
      url: 'https://www.npmjs.com/package/html-webpack-plugin',
      status: i18n[lang]['compatible'],
      description: i18n[lang]['html-webpack-plugin-desc'],
    },
    {
      name: '@sentry/webpack-plugin',
      url: 'https://www.npmjs.com/package/@sentry/webpack-plugin',
      status: i18n[lang]['compatible'],
      description: i18n[lang]['sentry_webpack-plugin-desc'],
    },
    {
      name: 'copy-webpack-plugin',
      url: 'https://www.npmjs.com/package/copy-webpack-plugin',
      status: i18n[lang]['included'],
      description: i18n[lang]['copy-plugin-desc'],
    },
    {
      name: 'mini-css-extract-plugin',
      url: 'https://webpack.js.org/plugins/mini-css-extract-plugin',
      status: i18n[lang]['included'],
      description: i18n[lang]['mini-css-extract-plugin-desc'],
    },
    {
      name: 'terser-webpack-plugin',
      url: 'https://webpack.js.org/plugins/terser-webpack-plugin',
      status: i18n[lang]['included'],
      description: i18n[lang]['terser-webpack-plugin-desc'],
    },
    {
      name: 'progressPlugin',
      url: 'https://webpack.js.org/plugins/progress-plugin',
      status: i18n[lang]['included'],
      description: i18n[lang]['progress-plugin-desc'],
    },
    {
      name: 'webpack-bundle-analyzer',
      url: 'https://www.npmjs.com/package/webpack-bundle-analyzer',
      status: i18n[lang]['compatible'],
    },
    {
      name: 'tsconfig-paths-webpack-plugin',
      url: 'https://www.npmjs.com/package/tsconfig-paths-webpack-plugin',
      status: i18n[lang]['included'],
      description: i18n[lang]['tsconfig-paths-webpack-plugin-desc'],
    },
    {
      name: 'fork-ts-checker-webpack-plugin',
      url: 'https://github.com/TypeStrong/fork-ts-checker-webpack-plugin',
      status: i18n[lang]['compatible'],
    },
  ];
  return pluginList
    .sort((item1, item2) => {
      return item1.status > item2.status ? 1 : -1;
    })
    .map((item) => <CompatibleCardItem key={item.name} {...item} />);
};

export const LoaderCompatibleCardList = ({ lang }: { lang: 'zh' | 'en' }) => {
  const loaderList: CardMeta[] = [
    {
      name: 'babel-loader',
      url: 'https://github.com/babel/babel-loader',
      status: i18n[lang]['compatible'],
      description: i18n[lang]['babel-loader-description'],
    },
    {
      name: 'sass-loader',
      url: 'https://github.com/webpack-contrib/sass-loader',
      status: i18n[lang]['compatible'],
    },
    {
      name: 'less-loader',
      url: 'https://github.com/webpack-contrib/less-loader',
      status: i18n[lang]['compatible'],
    },
    {
      name: 'postcss-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/postcss-loader',
    },
    {
      name: 'yaml-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/eemeli/yaml-loader',
    },
    {
      name: 'json-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/json-loader',
    },
    {
      name: 'stylus-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/stylus-loader',
    },
    {
      name: 'style-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/style-loader',
      description: i18n[lang]['style-loader-description'],
    },
    {
      name: 'file-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/file-loader',
    },
    {
      name: '@mdx-js/loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/mdx-js/mdx/tree/main/packages/loader',
    },
    {
      name: '@svgr/webpack',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/gregberge/svgr/tree/main/packages/webpack',
    },
    {
      name: 'raw-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/raw-loader',
    },
    {
      name: 'url-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/url-loader',
    },
    {
      name: 'css-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/css-loader',
    },
    {
      name: 'source-map-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/source-map-loader',
    },
    {
      name: 'thread-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/thread-loader',
    },
    {
      name: 'image-minimizer-webpack-plugin',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/webpack-contrib/image-minimizer-webpack-plugin',
    },
    {
      name: 'svg-react-loader',
      status: i18n[lang]['compatible'],
      url: 'https://github.com/jhamlet/svg-react-loader',
    },
    {
      name: 'node-loader',
      status: i18n[lang]['compatible'],
      url: 'https://www.npmjs.com/package/node-loader',
    },
  ];
  return loaderList.map((item) => (
    <CompatibleCardItem key={item.name} {...item} />
  ));
};
