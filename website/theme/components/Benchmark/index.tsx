import { motion } from 'framer-motion';
import { useState } from 'react';
import { useInView } from 'react-intersection-observer';
import { Tab, Tabs } from 'rspress/theme';
import { useI18n } from '../../i18n';
import { ProgressBar } from './ProgressBar';
import styles from './index.module.scss';

// 场景条件
// 冷启动/热更新
const BENCHMARK_DATA = {
  coldStart: [
    {
      name: 'Rspack',
      // 单位为 s
      time: 3.79,
    },
    {
      name: 'Webpack (with SWC)',
      time: 31.25,
    },
    {
      name: 'Webpack (with babel)',
      time: 42.61,
    },
  ],
  hmrRoot: [
    {
      name: 'Rspack',
      time: 0.57,
    },
    {
      name: 'Webpack (with SWC)',
      time: 1.67,
    },
    {
      name: 'Webpack (with babel)',
      time: 1.74,
    },
  ],
  hmrLeaf: [
    {
      name: 'Rspack',
      time: 0.56,
    },
    {
      name: 'Webpack (with SWC)',
      time: 1.53,
    },
    {
      name: 'Webpack (with babel)',
      time: 1.63,
    },
  ],
  coldBuild: [
    {
      name: 'Rspack',
      time: 22.35,
    },
    {
      name: 'Webpack (with SWC)',
      time: 75.05,
    },
    {
      name: 'Webpack (with babel + terser)',
      time: 160.06,
    },
  ],
};

const MODULE_COUNT_MAP = {
  coldStart: '50,000',
  hmrRoot: '10,000',
  hmrLeaf: '10,000',
  coldBuild: '50,000',
};

export function Benchmark() {
  const t = useI18n();
  const SCENE = ['coldStart', 'hmrRoot', 'hmrLeaf', 'coldBuild'];
  const [activeScene, setActiveScene] =
    useState<keyof typeof BENCHMARK_DATA>('coldStart');
  const { ref, inView } = useInView();
  const variants = {
    initial: { y: 50, opacity: 0 },
    animate: { y: 0, opacity: 1, transition: { duration: 0.5 } },
  };
  const performanceInfoList = BENCHMARK_DATA[activeScene];
  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 50 }}
      animate={inView ? 'animate' : 'initial'}
      variants={variants}
      transition={{ duration: 1 }}
      className="relative flex flex-col justify-center pt-20 pb-10 mt-15 h-auto"
    >
      {inView && (
        <>
          <div className="flex flex-center flex-col">
            <h2 className={`${styles.title} font-bold text-2xl sm:text-4xl`}>
              {t('benchmarkTitle')}
            </h2>
            <p className="mt-6 mb-3 mx-6 text-center sm:text-lg text-gray-500 max-w-3xl">
              {t('benchmarkDesc')}
            </p>
          </div>
          <div className="flex flex-col items-center my-4 z-1">
            <Tabs
              values={SCENE.map(item => ({
                label: t(item as keyof typeof BENCHMARK_DATA),
              }))}
              onChange={index =>
                setActiveScene(SCENE[index] as keyof typeof BENCHMARK_DATA)
              }
            >
              {SCENE.map(scene => (
                <Tab key={scene}>
                  {performanceInfoList.map(info => (
                    <div
                      key={info.name}
                      className="flex flex-center justify-start m-4 flex-col sm:flex-row"
                    >
                      {inView && (
                        <>
                          <p
                            className="mr-2 mb-2 w-20 text-center text-gray-400"
                            style={{ minWidth: '180px' }}
                          >
                            {info.name}
                          </p>
                          <ProgressBar
                            value={info.time}
                            max={Math.max(
                              ...performanceInfoList.map(info => info.time),
                            )}
                          />
                        </>
                      )}
                    </div>
                  ))}
                </Tab>
              ))}
            </Tabs>
            <div>
              <p className="font-medium my-2 text-center text-lg text-gray-500">
                <span className=" font-normal">{t('moduleCount')}:</span>{' '}
                {MODULE_COUNT_MAP[activeScene]}
              </p>
              <a
                href="misc/benchmark.html"
                className="hover:text-brand transition-colors duration-300 text-14px font-medium text-gray-500 p-3"
              >
                👉 {t('benchmarkDetail')}
              </a>
            </div>
            {/* <div className="flex flex-center">
      <p className="mr-2 font-medium">{t('moduleCount')}</p>
      <MenuGroup defaultLabel={activeLevel.toString()}>
        {LEVEL.map((level) => (
          <div
            key={level}
            className="text-sm py-1 px-3 cursor-pointer hover:bg-mute hover:text-brand rounded-md"
          >
            <span>{level}</span>
          </div>
        ))}
      </MenuGroup>
    </div> */}
          </div>
        </>
      )}
    </motion.div>
  );
}
