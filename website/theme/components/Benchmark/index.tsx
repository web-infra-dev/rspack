import { motion } from 'framer-motion';
import { useInView } from 'react-intersection-observer';
import { useI18n } from '../../i18n';
import { ProgressBar } from './ProgressBar';
import styles from './index.module.scss';

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
  const variants = {
    initial: { y: 50, opacity: 0 },
    animate: { y: 0, opacity: 1, transition: { duration: 0.5 } },
  };

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 50 }}
      animate={inView ? 'animate' : 'initial'}
      variants={variants}
      transition={{ duration: 1 }}
      className="relative flex flex-col justify-center pt-24 pb-10 mt-15 h-auto"
    >
      {inView && (
        <>
          <div className="flex flex-center flex-col">
            <h2 className={`${styles.title} font-bold text-2xl sm:text-4xl`}>
              {t('benchmarkTitle')}
            </h2>
            <p
              className={`${styles.desc} mt-8 mb-5 mx-6 text-center sm:text-lg max-w-3xl`}
            >
              {t('benchmarkDesc')}
            </p>
          </div>
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
        </>
      )}
    </motion.div>
  );
}
