import { NoSSR } from 'rspress/runtime';
import { HomeLayout as BasicHomeLayout } from 'rspress/theme';
import { Benchmark } from '../components/Benchmark';
import { HomeFooter } from '../components/HomeFooter/index';

export function HomeLayout() {
  return (
    <BasicHomeLayout
      afterFeatures={
        <>
          <NoSSR>
            <Benchmark />
          </NoSSR>
          <HomeFooter />
        </>
      }
    />
  );
}
