import { Link } from '@rspress/core/theme-original';
import { FullyFeatured as BaseFullyFeatured } from '@rstack-dev/doc-ui/fully-featured';
import {
  containerStyle,
  descStyle,
  innerContainerStyle,
  titleAndDescStyle,
  titleStyle,
} from '@rstack-dev/doc-ui/section-style';
import { memo } from 'react';
import { useI18n, useI18nUrl } from '../../../i18n';
import arrow from './assets/arrow.svg';
import javascriptApi from './assets/javascriptApi.svg';
import layer from './assets/layer.svg';
import lightningcss from './assets/lightningcss.svg';
import loader from './assets/loader.svg';
import moduleFederation from './assets/moduleFederation.svg';
import parallel from './assets/parallel.svg';
import reload from './assets/reload.svg';
import server from './assets/server.svg';
import setting from './assets/setting.svg';
import swc from './assets/swc.svg';
import tree from './assets/tree.svg';

type Feature = {
  icon: string;
  title: string;
  description: string;
  link: string;
};

const FullyFeatured = memo(() => {
  const t = useI18n();
  const tUrl = useI18nUrl();

  const FeatureRow1: Feature[] = [
    {
      icon: arrow,
      title: 'Code Splitting',
      description: t('featureCodeSplitting'),
      link: tUrl('/guide/optimization/code-splitting'),
    },
    {
      icon: tree,
      title: 'Tree Shaking',
      description: t('featureTreeShaking'),
      link: tUrl('/guide/optimization/tree-shaking'),
    },
    {
      icon: layer,
      title: 'Plugins',
      description: t('featurePlugins'),
      link: tUrl('/guide/features/plugin'),
    },
    {
      icon: moduleFederation,
      title: 'Module Federation',
      description: t('featureModuleFederation'),
      link: tUrl('/guide/features/module-federation'),
    },
  ];

  const FeatureRow2: Feature[] = [
    {
      icon: setting,
      title: 'Asset Management',
      description: t('featureAssetManagement'),
      link: tUrl('/guide/features/asset-module'),
    },
    {
      icon: loader,
      title: 'Loaders',
      description: t('featureLoaders'),
      link: tUrl('/guide/features/loader'),
    },
    {
      icon: reload,
      title: 'HMR',
      description: t('featureHmr'),
      link: tUrl('/api/runtime-api/hmr'),
    },
    {
      icon: server,
      title: 'Dev Server',
      description: t('featureDevServer'),
      link: tUrl('/guide/features/dev-server'),
    },
  ];

  const FeatureRow3: Feature[] = [
    {
      icon: parallel,
      title: 'Parallel Builds',
      description: t('featureParallelBuilds'),
      link: tUrl('/api/javascript-api/#multicompiler'),
    },
    {
      icon: swc,
      title: 'SWC',
      description: t('featureSwc'),
      link: tUrl('/guide/features/builtin-swc-loader'),
    },
    {
      icon: lightningcss,
      title: 'Lightning CSS',
      description: t('featureLightningCss'),
      link: tUrl('/plugins/rspack/lightning-css-minimizer-rspack-plugin'),
    },
    {
      icon: javascriptApi,
      title: 'JavaScript API',
      description: t('featureJavaScriptApi'),
      link: tUrl('/api/javascript-api/index'),
    },
  ];

  return (
    <section className={containerStyle}>
      <div className={innerContainerStyle}>
        <div className={titleAndDescStyle}>
          <h1 className={titleStyle}>{t('fullyFeaturedTitle')}</h1>
          <p className={descStyle}>{t('fullyFeaturedDesc')}</p>
        </div>
        <BaseFullyFeatured
          featureRows={[FeatureRow1, FeatureRow2, FeatureRow3]}
          LinkComp={Link}
        />
      </div>
    </section>
  );
});

export default FullyFeatured;
