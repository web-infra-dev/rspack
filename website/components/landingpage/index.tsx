import { useDark } from 'rspress/runtime';
import BackgroundSvg from './Background.png';
import { Benchmark } from './Benchmark';
import BuiltWithRspack from './BuiltWithRspack';
import FullyFeatured from './FullyFeatured';
import Hero from './Hero';
import WhyRspack from './WhyRspack';
import styles from './index.module.scss';

const Background = () => {
  return (
    <img className={styles.background} src={BackgroundSvg} alt="background" />
  );
};

const LandingPage = () => {
  const isDark = useDark();
  return (
    <div className={styles.landingPage}>
      <Background />
      <Hero />
      <WhyRspack />
      <Benchmark />
      <FullyFeatured />
      <BuiltWithRspack />
    </div>
  );
};

export default LandingPage;
