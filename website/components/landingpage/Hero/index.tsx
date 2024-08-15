import { useDark } from 'rspress/runtime';
import { useI18n } from '../../../theme/i18n';
import styles from './index.module.scss';

const Hero = () => {
  const isDark = useDark();
  const t = useI18n();
  return (
    <div className={styles.hero}>
      <div className={styles.innerHero}>
        <img
          src={
            isDark
              ? 'https://assets.rspack.dev/rspack/rspack-logo.svg'
              : 'https://assets.rspack.dev/rspack/rspack-logo.svg'
          }
          alt="logo"
          className={styles.logo}
        />
        <h1 className={styles.title}>Rspack</h1>
        <p className={styles.subtitle}>{t('heroSlogan')}</p>
        <p className={styles.description}>{t('heroSubSlogan')}</p>
        <div className={styles.buttons}>
          <button className={styles.buttonPrimary} type="button">
            {t('getStarted')}
          </button>
          <button className={styles.buttonSecondary} type="button">
            {t('learnMore')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default Hero;
