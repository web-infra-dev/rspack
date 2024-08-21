import {
  Benchmark as BaseBenchmark,
  type BenchmarkData,
} from './BaseBenchmark';

// TODO: extract to @rstack-dev/doc-ui/benchmark
// import {
//   Benchmark as BaseBenchmark,
//   type BenchmarkData,
// } from '@rstack-dev/doc-ui/benchmark';
import { useI18n } from '../../../i18n';
import sharedStyles from '../shared.module.scss';
import styles from './index.module.scss';

// Benchmark data for different cases
// Unit: second
// From: https://github.com/rspack-contrib/performance-compare
const BENCHMARK_DATA: BenchmarkData = {
  rspack: {
    label: 'Rspack',
    metrics: [
      {
        time: 0.49,
        desc: 'dev',
      },
      {
        time: 0.36,
        desc: 'build',
      },
      {
        time: 0.09,
        desc: 'hmr',
      },
    ],
  },
  viteSwc: {
    label: 'Vite + SWC',
    metrics: [
      {
        time: 1.58,
        desc: 'dev',
      },
      {
        time: 1.37,
        desc: 'build',
      },
      {
        time: 0.05,
        desc: 'hmr',
      },
    ],
  },
  webpackSwc: {
    label: 'webpack + SWC',
    metrics: [
      {
        time: 2.4,
        desc: 'dev',
      },
      {
        time: 2.12,
        desc: 'build',
      },
      {
        time: 0.22,
        desc: 'hmr',
      },
    ],
  },
  webpackBabel: {
    label: 'webpack + Babel',
    metrics: [
      {
        time: 5.13,
        desc: 'dev',
      },
      {
        time: 6.47,
        desc: 'build',
      },
      {
        time: 0.22,
        desc: 'hmr',
      },
    ],
  },
};

export function Benchmark() {
  const t = useI18n();
  return (
    <div className={sharedStyles.container}>
      <div className={sharedStyles.titleAndDesc}>
        <h1 className={sharedStyles.title}>{t('benchmarkTitle')}</h1>
        <p className={sharedStyles.desc}>{t('benchmarkDesc')}</p>
      </div>
      <BaseBenchmark data={BENCHMARK_DATA} />
      <div className="flex flex-col items-center self-stretch">
        <a
          href="https://github.com/rspack-contrib/performance-compare"
          target="_blank"
          className={styles.button}
          rel="noreferrer"
        >
          {t('benchmarkDetail')}
        </a>
      </div>
    </div>
  );
}
