import { useDark } from 'rspress/runtime';
import styles from './index.module.scss';

const Hero = () => {
  const isDark = useDark();
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
        <p className={styles.subtitle}>The fast Rust-based web bundler</p>
        <p className={styles.description}>
          Seamlessly replace webpack with compatible API
        </p>
        <div className={styles.buttons}>
          <button className={styles.buttonPrimary} type="button">
            Get Started
          </button>
          <button className={styles.buttonSecondary} type="button">
            Learn More
          </button>
        </div>
      </div>
    </div>
  );
};

export default Hero;
