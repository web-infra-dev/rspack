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
  return (
    <div className={styles.featuresContainer}>
      <div className={styles.featuresContainerInner}>
        <div className={styles.features}>
          <div className={`${styles.featureCard} ${styles.whyRspack}`}>
            <div className={styles.featureContent}>
              <h3 className={styles.whyRspackText}>{t('whyRspack')}</h3>
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
        </div>
        <div className={styles.features}>
          <Link
            className={styles.featureCard}
            href={tUrl('/guide/start/introduction')}
          >
            <div className={styles.featureIcon}>
              <img src={Lightning} alt="Lightning" />
            </div>
            <div className={styles.featureContent}>
              <h3 className={styles.featureTitle}>{t('LightningHMR')}</h3>
              <p className={styles.featureDescription}>
                {t('LightningHMRDesc')}
              </p>
            </div>
          </Link>

          <Link className={styles.featureCard} href={tUrl('/guide/tech/react')}>
            <div className={styles.featureIcon}>
              <img src={FrameCheck} alt="FrameWork" />
            </div>
            <div className={styles.featureContent}>
              <h3 className={styles.featureTitle}>{t('FrameworkAgnostic')}</h3>
              <p className={styles.featureDescription}>
                {t('FrameworkAgnosticDesc')}
              </p>
            </div>
          </Link>
          <Link
            className={styles.featureCard}
            href={tUrl('/guide/compatibility/plugin')}
          >
            <div className={styles.featureIcon}>
              <img src={Compatible} alt="Compatible" />
            </div>
            <div className={styles.featureContent}>
              <h3 className={styles.featureTitle}>{t('WebpackCompatible')}</h3>
              <p className={styles.featureDescription}>
                {t('WebpackCompatibleDesc')}
              </p>
            </div>
          </Link>
        </div>
      </div>
    </div>
  );
};

export default Features;
