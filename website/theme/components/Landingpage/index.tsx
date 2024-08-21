import BackgroundUrl from './Background.compressed.png';
import { Benchmark } from './Benchmark';
import BuiltWithRspack from './BuiltWithRspack';
import FullyFeatured from './FullyFeatured';
import Hero from './Hero';
import ToolStack from './ToolStack';
import WhyRspack from './WhyRspack';
import styles from './index.module.scss';

const Background = () => {
  return (
    <img className={styles.background} src={BackgroundUrl} alt="background" />
  );
};

const LandingPage = () => {
  return (
    <div className={styles.landingPage}>
      <style>
        {`:root {
              --rp-c-bg: #0b0c0e;
          }
          :root:not(.dark) {
              --rp-c-bg: #fff;
          }
          .rspress-nav {
            background: transparent !important;
          }`}
      </style>
      <Background />
      <Hero />
      <WhyRspack />
      <Benchmark />
      <FullyFeatured />
      <ToolStack />
      <BuiltWithRspack />
    </div>
  );
};

export default LandingPage;
