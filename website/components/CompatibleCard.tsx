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
  return loaderList.map(item => (
    <CompatibleCardItem key={item.name} {...item} />
  ));
};
