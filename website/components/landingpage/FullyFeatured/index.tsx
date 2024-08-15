import React, { useState } from 'react';
import { useLang } from 'rspress/runtime';
import { useI18n } from '../../../theme/i18n';
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
import styles from './index.module.scss';

type Feature = {
  icon: string;
  title: string;
  description: string;
};

const FullyFeatured = () => {
  const t = useI18n();
  const FeatureRow1: Feature[] = [
    {
      icon: arrow,
      title: 'Code Splitting',
      description: t('featureCodeSplitting'),
    },
    {
      icon: tree,
      title: 'Tree Shaking',
      description: t('featureTreeShaking'),
    },
    {
      icon: layer,
      title: 'Plugins',
      description: t('featurePlugins'),
    },
    {
      icon: moduleFederation,
      title: 'Module Federation',
      description: t('featureModuleFederation'),
    },
  ];

  const FeatureRow2: Feature[] = [
    {
      icon: setting,
      title: 'Asset Management',
      description: t('featureAssetManagement'),
    },
    {
      icon: loader,
      title: 'Loaders',
      description: t('featureLoaders'),
    },
    {
      icon: reload,
      title: 'HMR',
      description: t('featureHmr'),
    },
    {
      icon: server,
      title: 'Dev Server',
      description: t('featureDevServer'),
    },
  ];

  const FeatureRow3: Feature[] = [
    {
      icon: parallel,
      title: 'Parallel Builds',
      description: t('featureParallelBuilds'),
    },
    {
      icon: swc,
      title: 'SWC',
      description: t('featureSwc'),
    },
    {
      icon: lightningcss,
      title: 'Lightning CSS',
      description: t('featureLightningCss'),
    },
    {
      icon: javascriptApi,
      title: 'JavaScript API',
      description: t('featureJavaScriptApi'),
    },
  ];

  const [isFolded, setIsFolded] = useState(true);

  return (
    <div className={styles.container}>
      <div className={styles.content}>
        <h1 className={styles.title}>{t('fullyFeaturedTitle')}</h1>
        <p className={styles.subtitle}>{t('fullyFeaturedDesc')}</p>
      </div>
      <div className={styles.main}>
        {[FeatureRow1, FeatureRow2, ...(isFolded ? [] : [FeatureRow3])].map(
          (row, index) => {
            return (
              <div className={styles.features} key={index}>
                {row.map((feature, index) => (
                  <div key={index} className={styles.featureCard}>
                    <img
                      src={feature.icon}
                      alt={index.toString()}
                      className={styles.icon}
                    />
                    <div className={styles.featureContent}>
                      <h2 className={styles.featureTitle}>{feature.title}</h2>
                      <p className={styles.featureDescription}>
                        {feature.description}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            );
          },
        )}
        <button
          type="button"
          className={styles.button}
          onClick={() => {
            setIsFolded(folded => !folded);
          }}
        >
          {isFolded ? t('fullyfeaturedDetail') : t('foldFullyfeaturedDetail')}
        </button>
      </div>
    </div>
  );
};

export default FullyFeatured;
