import { useDark } from 'rspress/runtime';
import Features from './Features';
import Hero from './Hero';
import styles from './index.module.scss';

const LandingPage = () => {
  const isDark = useDark();
  return (
    <div className={styles.landingPage}>
      <Hero />
      <Features />
    </div>
  );
};

export default LandingPage;
