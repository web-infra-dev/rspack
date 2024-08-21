import { Link } from 'rspress/theme';
import { useI18n } from '../../../i18n';
import sharedStyles from '../shared.module.scss';
import amazonLogo from './assets/amazon.svg';
import bitDevLogo from './assets/bit.svg';
import bytedanceLogo from './assets/bytedance.svg';
import discordLogo from './assets/discord.svg';
import intuitLogo from './assets/intuit.svg';
import microsoftLogo from './assets/microsoft.svg';
import styles from './index.module.scss';

const BuiltWithRsPack: React.FC = () => {
  const t = useI18n();
  return (
    <div className={sharedStyles.container}>
      <div
        className={`${sharedStyles.innerContainer} ${styles.innerContainer}`}
      >
        <h2 className={styles.title}>{t('builtWithRspack')}</h2>
        <div className={styles.logos}>
          <Link
            className={`${styles.logo} ${styles.bitDevContainer}`}
            href="https://bit.dev/"
          >
            <img src={bitDevLogo} alt="bit.dev" className={styles.bitDevLogo} />
            <span className={styles.bitDevText}>bit.dev</span>
          </Link>
          <Link className={`${styles.logo}`} href="https://www.microsoft.com">
            <img
              src={microsoftLogo}
              alt="Microsoft"
              className={styles.microsoftLogo}
            />
          </Link>
          <Link className={styles.logo} href="https://amazon.com/">
            <img src={amazonLogo} alt="Amazon" className={styles.amazonLogo} />
          </Link>
          <Link className={styles.logo} href="https://www.bytedance.com">
            <img
              src={bytedanceLogo}
              alt="ByteDance"
              className={styles.bytedanceLogo}
            />
          </Link>
          <Link className={styles.logo} href="https://www.intuit.com">
            <img src={intuitLogo} alt="Intuit" className={styles.inituitLogo} />
          </Link>
          <Link className={styles.logo} href="https://discord.com">
            <img
              src={discordLogo}
              alt="discord"
              className={styles.discordLogo}
            />
          </Link>
        </div>
      </div>
    </div>
  );
};

export default BuiltWithRsPack;
