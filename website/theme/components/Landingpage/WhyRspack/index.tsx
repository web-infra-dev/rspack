import { Link } from 'rspress/theme';
import { useI18n, useI18nUrl } from '../../../i18n';
import Compatible from './assets/Compatible.svg';
import FrameCheck from './assets/FrameCheck.svg';
import Lightning from './assets/Lightning.svg';
import Speedometer from './assets/Speedometer.svg';
import WhyRspackBg from './assets/WhyRspackBg.png';
import styles from './index.module.scss';
import { useCardAnimation } from './useCardAnimation';

type Feature = {
  img: string;
  url: string;
  title: string;
  description: string;
};

const WhyRspackCard = () => {
  const t = useI18n();
  const {
    container,
    onMouseEnter,
    onMouseLeave,
    onMouseMove,
    onTouchEnd,
    onTouchMove,
    onTouchStart,
    outerContainer,
    ref,
    shine,
    shineBg,
  } = useCardAnimation();

  return (
    <div
      style={{
        position: 'relative',
        transform: outerContainer,
        transformStyle: 'preserve-3d',
        zIndex: 6,
        WebkitTapHighlightColor: 'rgba(#000, 0)',
      }}
      className={styles.whyRspackCard}
      ref={ref as any}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      onMouseMove={onMouseMove}
      onTouchMove={onTouchMove}
      onTouchEnd={onTouchEnd}
      onTouchStart={onTouchStart}
    >
      <div
        className={styles.whyRspack}
        style={{
          transform: container,
          position: 'relative',
          transition: 'all 0.2s ease-out',
        }}
      >
        <div
          className="shine"
          style={{
            position: 'absolute',
            top: '0',
            left: '0',
            right: '0',
            bottom: '0',
            borderRadius: '20px',
            zIndex: '8',
            ...(shine
              ? {
                  transform: shine,
                }
              : {}),
            ...(shineBg
              ? {
                  background: shineBg,
                }
              : {}),
          }}
        />
        <div className={styles.whyRspackContent}>
          <h3 className={styles.whyRspackTitle}>{t('whyRspack')}</h3>
          <p className={styles.whyRspackDescription}>{t('whyRspackDesc')}</p>
          <img className={styles.whyRspackBg} src={WhyRspackBg} alt="bg" />
        </div>
      </div>
    </div>
  );
};

const FeatureItem = ({ img, url, title, description }: Feature) => {
  const {
    container,
    onMouseEnter,
    onMouseLeave,
    onMouseMove,
    onTouchEnd,
    onTouchMove,
    onTouchStart,
    outerContainer,
    ref,
    shine,
    shineBg,
  } = useCardAnimation();

  return (
    <div
      style={{
        position: 'relative',
        transform: outerContainer,
        cursor: 'pointer',
        transformStyle: 'preserve-3d',
        WebkitTapHighlightColor: 'rgba(#000, 0)',
      }}
      className={styles.featureCard}
      ref={ref as any}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
      onMouseMove={onMouseMove}
      onTouchMove={onTouchMove}
      onTouchEnd={onTouchEnd}
      onTouchStart={onTouchStart}
    >
      <Link
        href={url}
        key={title}
        className={styles.featureCardInner}
        style={{
          transform: container,
          position: 'relative',
          transition: 'all 0.2s ease-out',
        }}
      >
        <div
          className="shine"
          style={{
            position: 'absolute',
            top: '0',
            left: '0',
            right: '0',
            bottom: '0',
            borderRadius: '20px',
            zIndex: '8',
            ...(shine
              ? {
                  transform: shine,
                }
              : {}),
            ...(shineBg
              ? {
                  background: shineBg,
                }
              : {}),
          }}
        />
        <div className={styles.featureIcon}>
          <img src={img} alt="Lightning" />
        </div>
        <div className={styles.featureContent}>
          <h3 className={styles.featureTitle}>{title}</h3>
          <p className={styles.featureDescription}>{description}</p>
        </div>
      </Link>
    </div>
  );
};

const WhyRspack = () => {
  const t = useI18n();
  const tUrl = useI18nUrl();

  const features: Feature[] = [
    {
      img: Speedometer,
      url: tUrl('/guide/start/introduction'),
      title: t('FastStartup'),
      description: t('FastStartupDesc'),
    },
    {
      img: Lightning,
      url: tUrl('/guide/start/introduction'),
      title: t('LightningHMR'),
      description: t('LightningHMRDesc'),
    },
    {
      img: FrameCheck,
      url: tUrl('/guide/tech/react'),
      title: t('FrameworkAgnostic'),
      description: t('FrameworkAgnosticDesc'),
    },
    {
      img: Compatible,
      url: tUrl('/guide/compatibility/plugin'),
      title: t('WebpackCompatible'),
      description: t('WebpackCompatibleDesc'),
    },
  ];

  return (
    <section className={styles.container}>
      <div className={styles.innerContainer}>
        <div className={styles.features}>
          {/* Why Rspack? */}
          <WhyRspackCard />
          {features.map(({ img, url, title, description }) => {
            return (
              <FeatureItem
                key={title}
                img={img}
                url={url}
                title={title}
                description={description}
              />
            );
          })}
        </div>
      </div>
    </section>
  );
};

export default WhyRspack;
