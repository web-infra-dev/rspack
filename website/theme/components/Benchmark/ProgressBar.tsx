import { motion } from 'framer-motion';
import { useState } from 'react';
import styles from './ProgressBar.module.scss';

export function formatTime(time: number, totalTime: number) {
  if (totalTime < 1000) {
    return `${time.toFixed(0)}ms`;
  } else {
    return `${(time / 1000).toFixed(2)}s`;
  }
}

export function ProgressBar({
  value,
  max,
  color,
  desc,
}: {
  value: number;
  max: number;
  color: string;
  desc: string;
}) {
  const [elapsedTime, setElapsedTime] = useState(0);
  const TOTAL_TIME = value * 1000;
  const isMobile = window.innerWidth < 768;
  const progressBarWidth = isMobile ? 80 : 55;
  const variants = {
    initial: { width: 0 },
    animate: { width: `${(value / max) * 100}%` },
  };

  const formattedTime = formatTime(elapsedTime, TOTAL_TIME);
  return (
    <div
      className={`${styles.container} flex items-center sm:pr-4`}
      style={{
        width: `${progressBarWidth}vw`,
      }}
    >
      <div className={`${styles['inner-container']} flex justify-between`}>
        <motion.div
          className={`${styles.bar} ${styles[color]}`}
          initial="initial"
          animate="animate"
          variants={variants}
          onUpdate={(latest: { width: string }) => {
            const width = Number.parseFloat(latest.width);
            setElapsedTime(width * max * 10);
          }}
          transition={{ duration: value, ease: 'linear' }}
        />
      </div>
      <div className={styles.desc}>
        <span className={styles.time}>{formattedTime}</span> {desc}
      </div>
    </div>
  );
}
