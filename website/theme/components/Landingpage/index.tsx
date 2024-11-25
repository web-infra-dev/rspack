import { BackgroundImage } from '@rstack-dev/doc-ui/background-image';
import { Benchmark } from './Benchmark';
import FullyFeatured from './FullyFeatured';
import Hero from './Hero';
import ToolStack from './ToolStack';
import WhoIsUsing from './WhoIsUsing';
import WhyRspack from './WhyRspack';
import styles from './index.module.scss';

const LandingPage = () => {
  return (
    <div className={styles.landingPage}>
      <BackgroundImage />
      <Hero />
      <WhyRspack />
      <Benchmark />
      <FullyFeatured />
      <ToolStack />
      <WhoIsUsing />
    </div>
  );
};

export default LandingPage;
