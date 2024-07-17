import { useI18n, useUrl } from '../../i18n';
import styles from './index.module.scss';

export function Featured() {
  const t = useI18n();

  const features = [
    {
      title: 'Code Splitting',
      desc: t('featureCodeSplitting'),
      link: useUrl('/guide/optimization/code-splitting'),
    },
    {
      title: 'Plugins',
      desc: t('featurePlugins'),
      link: useUrl('/guide/features/plugin'),
    },
    {
      title: 'HMR',
      desc: t('featureHmr'),
      link: useUrl('/api/hmr'),
    },
    {
      title: 'SWC',
      desc: t('featureSwc'),
      link: useUrl('/guide/features/builtin-swc-loader'),
    },
    {
      title: 'Tree Shaking',
      desc: t('featureTreeShaking'),
      link: useUrl('/guide/optimization/tree-shaking'),
    },
    {
      title: 'Loaders',
      desc: t('featureLoaders'),
      link: useUrl('/guide/features/loader'),
    },
    {
      title: 'Dev Server',
      desc: t('featureDevServer'),
      link: useUrl('/guide/features/dev-server'),
    },
    {
      title: 'Lightning CSS',
      desc: t('featureLightningCss'),
      link: useUrl('/plugins/rspack/lightning-css-minimizer-rspack-plugin'),
    },
    {
      title: 'Module Federation',
      desc: t('featureModuleFederation'),
      link: useUrl('/guide/features/module-federation'),
    },
    {
      title: 'Asset Management',
      desc: t('featureAssetManagement'),
      link: useUrl('/guide/features/asset-module'),
    },
    {
      title: 'Parallel Builds',
      desc: t('featureParallelBuilds'),
      link: useUrl('/api/javascript-api#multicompiler'),
    },
    {
      title: 'JavaScript API',
      desc: t('featureJavaScriptApi'),
      link: useUrl('/api/javascript-api'),
    },
  ];

  return (
    <div className="relative flex flex-center flex-col justify-center pt-24 pb-10 mt-10 h-auto">
      {
        <>
          <div className="flex flex-center flex-col">
            <h2
              className={`${styles.title} font-bold text-3xl sm:text-5xl mt-16`}
            >
              {t('featuredTitle')}
            </h2>
            <p
              className={`${styles.desc} mt-8 mb-5 mx-6 text-center text-lg max-w-3xl`}
            >
              {t('featuredDesc')}
            </p>
          </div>
          <div className={styles.list}>
            {features.map(feature => {
              return (
                <a href={feature.link} key={feature.title}>
                  <div className={styles.card}>
                    <div className={styles.cardTitle}>{feature.title}</div>
                    <div className={styles.cardDesc}>{feature.desc}</div>
                  </div>
                </a>
              );
            })}
          </div>
        </>
      }
    </div>
  );
}
