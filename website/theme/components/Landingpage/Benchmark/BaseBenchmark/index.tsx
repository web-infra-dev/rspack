import { useInView } from 'react-intersection-observer';
import { ProgressBar } from './ProgressBar';
import styles from './index.module.scss';

export type BenchmarkData = Record<
  string,
  {
    label: string;
    metrics: Array<{ time: number; desc: string }>;
  }
>;

export interface BenchmarkProps {
  data: BenchmarkData;
}

const findMaxTime = (data: BenchmarkData) => {
  let max = 0;

  for (const item of Object.values(data)) {
    for (const metric of item.metrics) {
      if (metric.time > max) {
        max = metric.time;
      }
    }
  }

  return max;
};

export function Benchmark({ data }: BenchmarkProps) {
  const maxTime = findMaxTime(data);
  const { ref, inView } = useInView({
    triggerOnce: true,
  });
  return (
    <div className={styles.root} ref={ref}>
      {Object.values(data).map(item => (
        <div key={item.label} className={styles.item}>
          <p className={styles.progressName}>{item.label}</p>
          <div className={styles.metricItem}>
            {item.metrics.map(metric => (
              <ProgressBar
                key={metric.desc}
                value={metric.time}
                max={maxTime}
                desc={metric.desc}
                inView={inView}
              />
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}
