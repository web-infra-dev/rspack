import { motion } from 'framer-motion';
import { useState } from 'react';
import styles from './ProgressBar.module.scss';

export function formatTime(time: number, totalTime: number) {
  if (totalTime < 1000) {
    return `${time.toFixed(0)}ms`;
  }
  return `${(time / 1000).toFixed(2)}s`;
}

export function ProgressBar({
  value,
  max,
  desc,
  inView,
}: {
  value: number;
  max: number;
  desc: string;
  inView: boolean;
}) {
  const [elapsedTime, setElapsedTime] = useState(0);
  const TOTAL_TIME = value * 1000;
  const variants = {
    initial: { width: 0 },
    animate: { width: `${(value / max) * 100}%` },
  };

  const formattedTime = formatTime(elapsedTime, TOTAL_TIME);
  return (
    <div className={styles.container}>
      <div className={styles.innerContainer}>
        {inView ? (
          <motion.div
            className={styles.bar}
            initial="initial"
            animate="animate"
            variants={variants}
            onUpdate={(latest: { width: string }) => {
              const width = Number.parseFloat(latest.width);
              setElapsedTime(width * max * 10);
            }}
            transition={{ duration: value, ease: 'linear' }}
          />
        ) : null}
      </div>
      <div className={styles.desc}>
        <span className={styles.time}>{formattedTime}</span> {desc}
      </div>
    </div>
  );
}
