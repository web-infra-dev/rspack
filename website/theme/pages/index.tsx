import { NoSSR, usePageData } from 'rspress/runtime';
import { Benchmark } from '../components/Benchmark';
import { type Feature, HomeFeature } from '../components/HomeFeatures';
import { HomeFooter } from '../components/HomeFooter/index';
import { type Hero, HomeHero } from '../components/HomeHero';

export function HomeLayout() {
  const { page } = usePageData();
  const { frontmatter } = page;
  return (
    <div>
      {/* Landing Page */}
      <div
        className="relative border-b"
        style={{
          paddingBottom: '56px',
          borderColor: 'var(--rp-c-divider-light)',
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
