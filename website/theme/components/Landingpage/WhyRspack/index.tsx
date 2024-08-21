import { Link } from 'rspress/theme';
import { useI18n, useI18nUrl } from '../../../i18n';
import Compatible from './assets/Compatible.svg';
import FrameCheck from './assets/FrameCheck.svg';
import Lightning from './assets/Lightning.svg';
import Speedometer from './assets/Speedometer.svg';
import WhyRspackBg from './assets/WhyRspackBg.png';
import styles from './index.module.scss';

const Features = () => {
  const t = useI18n();
  const tUrl = useI18nUrl();

  const features = [
    {
      img: Speedometer,
      url: tUrl('/guide/start/introduction'),
      title: t('FastStartup'),
      description: t('FastStartupDesc'),
    },
    {
      img: Lightning,
      url: tUrl('/guide/start/introduction'),
      title: t('LightningHMR'),
      description: t('LightningHMRDesc'),
    },
    {
      img: FrameCheck,
      url: tUrl('/guide/tech/react'),
      title: t('FrameworkAgnostic'),
      description: t('FrameworkAgnosticDesc'),
    },
    {
      img: Compatible,
      url: tUrl('/guide/compatibility/plugin'),
      title: t('WebpackCompatible'),
      description: t('WebpackCompatibleDesc'),
    },
  ];

  return (
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.features}>
          {/* Why Rspack? */}
          <div className={styles.whyRspack}>
            <div className={styles.whyRspackContent}>
              <h3 className={styles.whyRspackTitle}>{t('whyRspack')}</h3>
              <p className={styles.whyRspackDescription}>
                {t('whyRspackDesc')}
              </p>
              <img className={styles.whyRspackBg} src={WhyRspackBg} alt="bg" />
            </div>
          </div>
          <Link
            className={styles.featureCard}
            href={tUrl('/guide/start/introduction')}
          >
            <div className={styles.featureIcon}>
              <img src={Speedometer} alt="Speedometer" />
            </div>
            <div className={styles.featureContent}>
              <h3 className={styles.featureTitle}>{t('FastStartup')}</h3>
              <p className={styles.featureDescription}>
                {t('FastStartupDesc')}
              </p>
            </div>
          </Link>
          {features.slice(1).map(({ img, url, title, description }) => {
            return (
              <Link className={styles.featureCard} href={url} key={title}>
                <div className={styles.featureIcon}>
                  <img src={img} alt="Lightning" />
                </div>
                <div className={styles.featureContent}>
                  <h3 className={styles.featureTitle}>{title}</h3>
                  <p className={styles.featureDescription}>{description}</p>
                </div>
              </Link>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default Features;
