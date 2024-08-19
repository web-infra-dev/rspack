import { useCallback } from 'react';
import { useNavigate } from 'rspress/runtime';
import { useI18n, useI18nUrl } from '../../../i18n';
import styles from './index.module.scss';

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
    <div className={styles.hero}>
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
        <div className={styles.buttons}>
          <button
            className={styles.buttonPrimary}
            type="button"
            onClick={handleClickGetStarted}
          >
            {t('getStarted')}
          </button>
          <button
            className={styles.buttonSecondary}
            type="button"
            onClick={handleClickLearnMore}
          >
            {t('learnMore')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Hero;
