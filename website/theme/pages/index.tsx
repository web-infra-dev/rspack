// import { HomeLayout as BasicHomeLayout } from 'rspress/theme';
// import { Benchmark } from '../components/Benchmark';
// import { Featured } from '../components/Featured';
// import { HomeFooter } from '../components/HomeFooter/index';
// import { ToolStack } from '../components/ToolStack/ToolStack';
import LandingPage from '../../components/landingpage';

export function HomeLayout() {
  return (
    <LandingPage />
    // <BasicHomeLayout
    //   afterFeatures={
    //     <>
    //       <Benchmark />
    //       <Featured />
    //       <ToolStack />
    //       <HomeFooter />
    //     </>
    //   }
    // />
  );
}
