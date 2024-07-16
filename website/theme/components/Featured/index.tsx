import { useI18n } from '../../i18n';
import styles from './index.module.scss';

export function Featured() {
  const t = useI18n();

  const features = [
    {
      title: 'Code Splitting',
      desc: t('featureCodeSplitting'),
    },
    {
      title: 'Plugins',
      desc: t('featurePlugins'),
    },
    {
      title: 'HMR',
      desc: t('featureHmr'),
    },
    {
      title: 'SWC',
      desc: t('featureSwc'),
    },
    {
      title: 'Tree Shaking',
      desc: t('featureTreeShaking'),
    },
    {
      title: 'Loaders',
      desc: t('featureLoaders'),
    },
    {
      title: 'Dev Server',
      desc: t('featureDevServer'),
    },
    {
      title: 'Lightning CSS',
      desc: t('featureLightningCss'),
    },
    {
      title: 'Module Federation',
      desc: t('featureModuleFederation'),
    },
    {
      title: 'Asset Management',
      desc: t('featureAssetManagement'),
    },
    {
      title: 'Parallel Builds',
      desc: t('featureParallelBuilds'),
    },
    {
      title: 'JavaScript API',
      desc: t('featureJavaScriptApi'),
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
                <div className={styles.card}>
                  <div className={styles.cardTitle}>{feature.title}</div>
                  <div className={styles.cardDesc}>{feature.desc}</div>
                </div>
              );
            })}
          </div>
        </>
      }
    </div>
  );
}
