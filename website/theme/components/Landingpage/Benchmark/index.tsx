import {
  Benchmark as BaseBenchmark,
  type BenchmarkData,
} from '@rstack-dev/doc-ui/benchmark';
import {
  containerStyle,
  descStyle,
  innerContainerStyle,
  titleAndDescStyle,
  titleStyle,
} from '@rstack-dev/doc-ui/section-style';
import { memo } from 'react';
import { useI18n } from '../../../i18n';
import styles from './index.module.scss';

// Benchmark data for different cases
// Unit: second
// From: https://github.com/rstackjs/performance-compare
const BENCHMARK_DATA: BenchmarkData = {
  rspack: {
    label: 'Rspack',
    metrics: [
      {
        time: 0.41,
        desc: 'dev',
      },
      {
        time: 0.28,
        desc: 'build',
      },
      {
        time: 0.08,
        desc: 'hmr',
      },
    ],
  },
  viteSwc: {
    label: 'Vite + SWC',
    metrics: [
      {
        time: 1.29,
        desc: 'dev',
      },
      {
        time: 1.39,
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
        time: 2.26,
        desc: 'dev',
      },
      {
        time: 2.01,
        desc: 'build',
      },
      {
        time: 0.2,
        desc: 'hmr',
      },
    ],
  },
  webpackBabel: {
    label: 'webpack + Babel',
    metrics: [
      {
        time: 5.02,
        desc: 'dev',
      },
      {
        time: 6.52,
        desc: 'build',
      },
      {
        time: 0.2,
        desc: 'hmr',
      },
    ],
  },
};

export const Benchmark = memo(() => {
  const t = useI18n();
  return (
    <section className={containerStyle}>
      <div className={innerContainerStyle}>
        <div className={titleAndDescStyle}>
          <h1 className={titleStyle}>{t('benchmarkTitle')}</h1>
          <p className={descStyle}>{t('benchmarkDesc')}</p>
        </div>
        <BaseBenchmark data={BENCHMARK_DATA} />
        <div className="flex flex-col items-center self-stretch">
          <a
            href="https://github.com/rstackjs/performance-compare"
            target="_blank"
            className={styles.button}
            rel="noreferrer"
          >
            {t('benchmarkDetail')}
          </a>
        </div>
      </div>
    </section>
  );
});
