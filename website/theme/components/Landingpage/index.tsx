import { useEffect, useState } from 'react';
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
    <img
      className={styles.background}
      src="https://assets.rspack.dev/rspack/assets/landingpage-background-compressed.png"
      alt="background"
    />
  );
};

const useTopArrived = () => {
  const [scrollY, setScrollY] = useState(0);
  const topArrived = scrollY < 100;

  useEffect(() => {
    const handleScroll = () => {
      setScrollY(window.scrollY);
    };
    window.addEventListener('scroll', handleScroll, {
      capture: false,
      passive: true,
    });
    return () => {
      window.removeEventListener('scroll', handleScroll);
    };
  }, []);

  return {
    topArrived,
  };
};

const LandingPage = () => {
  const { topArrived } = useTopArrived();
  useEffect(() => {
    if (topArrived) {
      document.body.classList.remove('notTopArrived');
    } else {
      document.body.classList.add('notTopArrived');
    }
  }, [topArrived]);

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
            transition: background 0.4s;
          }
          body:not(.notTopArrived) .rspress-nav {
            background: transparent !important;
          }
          `}
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
