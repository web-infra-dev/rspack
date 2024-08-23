import { useCallback } from 'react';
import { useNavigate } from 'rspress/runtime';
import { useI18n, useI18nUrl } from '../../../i18n';
import BackgroundStar from './BackgroundStar';
import styles from './index.module.scss';

const positions = [
  [91.4, 22.9],
  [36, 67.6],
  [92.4, 59.2],
  [55.4, 96.7],
  [94.1, 47.7],
  [33.8, 32.5],
  [43.1, 77.6],
  [9.8, 73.4],
  [16.1, 17.1],
  [1, 10],
  [88.3, 44.9],
  [19.3, 79],
  [28.5, 11.8],
  [94.6, 44.9],
  [53.8, 3.4],
  [41.7, 20],
  [56.9, 80.6],
  [16.4, 83.4],
  [39.4, 66.8],
  [19, 18.4],
  [25.1, 28.1],
  [18.9, 35.6],
  [39.8, 95.4],
  [32.9, 12.3],
  [21.2, 72.8],
  [83.3, 79.8],
  [71.4, 36.8],
  [62.2, 50.4],
  [45.4, 13.7],
  [61.1, 28.5],
  [18.2, 64.1],
  [40.7, 34.7],
  [87.6, 20.7],
  [78.3, 30.8],
  [84.4, 58.2],
  [74.2, 74.5],
  [12.4, 66.4],
  [68.2, 41.6],
];

const stars = positions.map(([top, left], i) => {
  return (
    <BackgroundStar
      key={i}
      top={`${top}%`}
      left={`${left}%`}
      size={i / 40 + 3}
    />
  );
});

const Hero = () => {
  const tUrl = useI18nUrl();
  const t = useI18n();

  const navigate = useNavigate();
  const handleClickGetStarted = useCallback(() => {
    navigate(tUrl('/guide/start/quick-start'));
  }, [tUrl, navigate]);

  const handleClickLearnMore = useCallback(() => {
    navigate(tUrl('/guide/start/introduction'));
  }, [tUrl, navigate]);

  return (
    <section className={styles.hero}>
      {stars}
      <div className={styles.innerHero}>
        <div className={styles.logo}>
          <img
            src="https://assets.rspack.dev/rspack/rspack-logo.svg"
            alt="rspack-logo"
          />
          <div className={styles.ovalBg} />
        </div>
        <h1 className={styles.title}>Rspack</h1>
        <p className={styles.subtitle}>{t('heroSlogan')}</p>
        <p className={styles.description}>{t('heroSubSlogan')}</p>
        <div className={styles.buttonGroup}>
          <button
            className={`${styles.button} ${styles.buttonPrimary}`}
            type="button"
            onClick={handleClickGetStarted}
          >
            {t('getStarted')}
          </button>
          <button
            className={`${styles.button} ${styles.buttonSecondary}`}
            type="button"
            onClick={handleClickLearnMore}
          >
            {t('learnMore')}
          </button>
        </div>
      </div>
    </section>
  );
};

export default Hero;
