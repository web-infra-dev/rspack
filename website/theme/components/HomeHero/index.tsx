import { Button } from 'rspress/theme';
import { normalizeHrefInRuntime } from 'rspress/runtime';
import styles from './index.module.scss';

export interface Hero {
  name: string;
  text: string;
  tagline: string;
  image?: {
    src: string;
    alt: string;
  };
  actions: {
    text: string;
    link: string;
    theme: 'brand' | 'alt';
  }[];
}

export function HomeHero({ hero }: { hero: Hero }) {
  const hasImage = hero.image !== undefined;
  return (
    <div
      className="m-auto px-6 pb-12 sm:pt-0 sm:px-8  md:px-16 md:pb-16"
      style={{
        height: 'calc(100vh - var(--rp-nav-height)))',
      }}
    >
      <div className="max-w-6xl m-auto flex flex-col md:flex-row">
        <div className="m-auto flex flex-col order-2 md:order-1 justify-center text-center">
          <h1 className="text-3xl sm:text-6xl md:text-7xl font-bold pb-3 lg:pb-5 z-10">
            <span className={styles.clip}>{hero.name}</span>
          </h1>
          {hero.text?.length && (
            <p
              className={`pb-2 mx-auto md:m-0 text-3xl sm:text-5xl md:text-6xl font-bold z-10  max-w-xs sm:max-w-xl`}
              style={{ lineHeight: '1.15' }}
            >
              {hero.text}
            </p>
          )}

          <p className="pt-2 m-auto md:m-0 text-sm sm:text-xl md:text-2xl text-text-2 font-medium z-10 whitespace-pre-wrap">
            {hero.tagline}
          </p>
          <div className="justify-center gap-3 flex flex-wrap m--1.5 pt-4 z-10">
            {hero.actions.map(action => (
              <div key={action.link} className="p-1 flex-shrink-0">
                <Button
                  type="a"
                  text={action.text}
                  href={normalizeHrefInRuntime(action.link)}
                  theme={action.theme}
                />
              </div>
            ))}
          </div>
        </div>

        {hasImage ? (
          <div className="modern-doc-home-hero-image m-auto flex-center md:none lg:flex order-1 md:order-2">
            <div className={styles.imgMask}></div>
            <img
              src="https://assets.rspack.dev/rspack/rspack-logo.svg"
              alt={hero.image?.alt}
            />
          </div>
        ) : null}
      </div>
    </div>
  );
}
