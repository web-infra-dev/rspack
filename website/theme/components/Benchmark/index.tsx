import { useInView } from 'react-intersection-observer';
import { useI18n } from '../../i18n';
import { ProgressBar } from './ProgressBar';
import styles from './index.module.scss';
import { NoSSR } from 'rspress/runtime';

// Benchmark data for different cases
// Unit: second
// From: https://github.com/rspack-contrib/performance-compare
const BENCHMARK_DATA = {
  rspack: {
    label: 'Rspack',
    coldStart: 0.49,
    coldBuild: 0.36,
    hmr: 0.09,
  },
  webpackSwc: {
    label: 'webpack + SWC',
    coldStart: 2.4,
    coldBuild: 2.12,
    hmr: 0.22,
  },
  webpackBabel: {
    label: 'webpack + Babel',
    coldStart: 5.13,
    coldBuild: 6.47,
    hmr: 0.22,
  },
};
const maxTime = 6.47;

export function Benchmark() {
  const t = useI18n();
  const { ref, inView } = useInView();

  return (
    <div
      ref={ref}
      className="relative flex flex-col justify-center pt-24 pb-10 mt-18 h-auto"
    >
      <div className="flex flex-center flex-col">
        <h2 className={`${styles.title} font-bold text-3xl sm:text-5xl mt-16`}>
          {t('benchmarkTitle')}
        </h2>
        <p
          className={`${styles.desc} mt-8 mb-5 mx-6 text-center text-lg max-w-3xl`}
        >
          {t('benchmarkDesc')}
        </p>
      </div>
      {inView && (
        <NoSSR>
          <div className="flex flex-col items-center my-4 z-1">
            {Object.values(BENCHMARK_DATA).map(item => (
              <div
                key={item.label}
                className={`${styles.item} flex flex-center justify-start m-5`}
              >
                {inView && (
                  <>
                    <p className={styles.progressName}>{item.label}</p>
                    <div>
                      <ProgressBar
                        value={item.coldStart}
                        max={maxTime}
                        color="cyan"
                        desc="dev"
                      />
                      <ProgressBar
                        value={item.coldBuild}
                        max={maxTime}
                        color="blue"
                        desc="build"
                      />
                      <ProgressBar
                        value={item.hmr}
                        max={maxTime}
                        color="cyan"
                        desc="HMR"
                      />
                    </div>
                  </>
                )}
              </div>
            ))}
            <div>
              <a
                href="misc/benchmark.html"
                target="_blank"
                className={`${styles['bottom-link']} hover:text-brand transition-colors duration-300 font-medium p-2`}
              >
                👉 {t('benchmarkDetail')}
              </a>
            </div>
          </div>
        </NoSSR>
      )}
    </div>
  );
}
