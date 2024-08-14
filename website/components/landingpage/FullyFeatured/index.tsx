import React from 'react';
import { useLang } from 'rspress/runtime';
import { useI18n } from '../../../theme/i18n';
import arrow from './assets/arrow.svg';
import layer from './assets/layer.svg';
import loader from './assets/loader.svg';
import moduleFederation from './assets/moduleFederation.svg';
import reload from './assets/reload.svg';
import server from './assets/server.svg';
import setting from './assets/setting.svg';
import tree from './assets/tree.svg';
import styles from './index.module.scss';

const FullyFeatured = () => {
  const lang = useLang();
  const FeatureRow1 = [
    {
      icon: arrow,
      title: 'Code Splitting',
      description:
        'Split code into smaller bundles to enable on-demand loading and improve performance.',
    },
    {
      icon: tree,
      title: 'Tree Shaking',
      description:
        'Detect and eliminate unused code from the final bundles to reduce output size.',
    },
    {
      icon: layer,
      title: 'Plugins',
      description:
        'Offer rich plugin hooks and compatibility with most webpack plugins.',
    },
    {
      icon: moduleFederation,
      title: 'Module Federation',
      description:
        'Share code between web applications and collaborate more efficiently.',
    },
  ];

  const FeatureRow2 = [
    {
      icon: setting,
      title: 'Asset Management',
      description:
        'Handles and optimizes static assets like images, fonts and stylesheets.',
    },
    {
      icon: loader,
      title: 'Loaders',
      description:
        'Fully compatible with webpack loaders, reusing the entire ecosystem.',
    },
    {
      icon: reload,
      title: 'HMR',
      description:
        'Hot updating of modules at runtime without the need for a full refresh.',
    },
    {
      icon: server,
      title: 'Dev Server',
      description:
        'Provides a mature, high-performance dev server for local development.',
    },
  ];

  return (
    <div className={styles.container}>
      <div className={styles.content}>
        <h1 className={styles.title}>Fully Featured</h1>
        <p className={styles.subtitle}>
          Launched as a drop-in replacement for webpack, with more powerful
          features and exceptional productivity.
        </p>
      </div>
      <div className={styles.features}>
        {FeatureRow1.map((feature, index) => (
          <div key={index} className={styles.featureCard}>
            <img
              src={feature.icon}
              alt={index.toString()}
              className={styles.icon}
            />
            <div className={styles.featureContent}>
              <h2 className={styles.featureTitle}>{feature.title}</h2>
              <p className={styles.featureDescription}>{feature.description}</p>
            </div>
          </div>
        ))}
      </div>
      <div className={styles.features}>
        {FeatureRow2.map((feature, index) => (
          <div key={index} className={styles.featureCard}>
            <img
              src={feature.icon}
              alt={index.toString()}
              className={styles.icon}
            />
            <div className={styles.featureContent}>
              <h2 className={styles.featureTitle}>{feature.title}</h2>
              <p className={styles.featureDescription}>{feature.description}</p>
            </div>
          </div>
        ))}
      </div>
      <button type="button" className={styles.button}>
        See All Features
      </button>
    </div>
  );
};

export default FullyFeatured;
