import { useCallback } from 'react';
import { useNavigate } from 'rspress/runtime';
import { useI18n, useI18nUrl } from '../../../i18n';
import BackgroundStar from './BackgroundStar';
import styles from './index.module.scss';

const positions = [
  [78.1, 10],
  [74.5, 70.3],
  [7.5, 39.2],
  [72.7, 43.6],
  [29.7, 60.2],
  [31.2, 27.6],
  [42, 22],
  [68.3, 72],
  [76.5, 53],
  [15.4, 6.7],
  [4, 13],
  [31.2, 10.2],
  [38.8, 86.5],
  [3.7, 33.9],
  [24.6, 4.9],
  [17.7, 6.1],
  [31.9, 37.9],
  [36.7, 49.9],
  [41.8, 19.6],
  [25.2, 0.6],
  [30, 7.1],
  [39.3, 49.7],
  [51.9, 16.4],
  [0.5, 22],
  [75.8, 39.2],
  [38.2, 44.1],
  [76.6, 30.3],
  [28, 45.1],
  [65.1, 18.5],
  [35.8, 75],
  [49.4, 86.4],
  [14.5, 47.7],
  [66.7, 65.8],
  [17.9, 20.5],
  [8.1, 77.9],
  [68.3, 45.4],
  [34.2, 0],
  [31.9, 66.7],
  [38.9, 80.7],
  [27.4, 53.2],
  [68, 38.1],
  [37.1, 5.9],
  [35.8, 8.3],
  [37.7, 0.4],
  [54.9, 19.4],
  [17.2, 62.9],
  [47.3, 50.3],
  [16.4, 77.9],
  [42, 79.7],
  [7, 45.1],
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
