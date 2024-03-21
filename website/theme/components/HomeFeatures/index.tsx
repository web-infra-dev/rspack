import styles from './index.module.scss';
import { motion } from 'framer-motion';
const PRESET_COUNT = [2, 3, 4];

const getGridClass = (count?: number): string => {
  if (!count) {
    return '';
  } else if (PRESET_COUNT.includes(count)) {
    return `grid-${12 / count}`;
  } else if (count % 3 === 0) {
    return 'grid-4';
  } else if (count % 2 === 0) {
    return 'grid-6';
  }
  return '';
};

export interface Feature {
  icon: string;
  title: string;
  details: string;
  link?: string;
}

export function HomeFeature({ features }: { features: Feature[] }) {
  const gridClass = getGridClass(features?.length);

  return (
    <div className={styles.featureContainer}>
      {features?.map((feature, index) => {
        const { icon, title, details, link } = feature;
        return (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, delay: index * 0.1 }}
            key={title}
            className={`${
              gridClass ? styles[gridClass] : 'w-full'
            } rounded-md hover:var(--rp-c-brand) mb-6`}
          >
            <div className="p-2 h-full">
              <article
                key={title}
                className={styles.featureCard}
                style={{
                  cursor: link ? 'pointer' : 'auto',
                }}
                onClick={() => {
                  if (link) {
                    window.location.href = link;
                  }
                }}
              >
                <div className="flex-center">
                  <div className="w-12 h-12 text-3xl text-center">{icon}</div>
                </div>
                <h2 className="font-bold text-center">{title}</h2>
                <p className="leading-6 pt-2 text-sm text-text-2 font-medium">
                  {details}
                </p>
              </article>
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}
