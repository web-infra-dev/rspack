import { Hero, HomeHero } from '../components/HomeHero';
import { HomeFeature, Feature } from '../components/HomeFeatures';
import { NoSSR, usePageData } from 'rspress/runtime';
import { Benchmark } from '../components/Benchmark';
import { HomeFooter } from '../components/HomeFooter/index';

export function HomeLayout() {
  const { page } = usePageData();
  const { frontmatter } = page;
  return (
    <div>
      {/* Landing Page */}
      <div
        className="relative border-b dark:border-dark-50"
        style={{
          background: 'var(--rp-home-bg)',
          minHeight: 'calc(80rem - var(--rp-nav-height))',
          paddingBottom: '56px',
        }}
      >
        <div className="pt-14 pb-12">
          <HomeHero hero={frontmatter.hero as Hero} />
          <HomeFeature features={frontmatter.features as Feature[]} />
        </div>
      </div>
      {/* Benchmark Page */}
      <NoSSR>
        <Benchmark />
      </NoSSR>
      {/* Footer */}
      <HomeFooter />
    </div>
  );
}
