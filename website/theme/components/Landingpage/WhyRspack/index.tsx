import { memo, useMemo } from 'react';
import { Link } from 'rspress/theme';
import { useI18n, useI18nUrl } from '../../../i18n';
import sharedStyles from '../shared.module.scss';
import CompatibleJson from './assets/Compatible.json';
import Compatible from './assets/Compatible.svg';
import FrameCheckJson from './assets/FrameCheck.json';
import FrameCheck from './assets/FrameCheck.svg';
import LightningJson from './assets/Lightning.json';
import Lightning from './assets/Lightning.svg';
import SpeedometerJson from './assets/Speedometer.json';
import Speedometer from './assets/Speedometer.svg';
import styles from './index.module.scss';
import { useCardAnimation } from './useCardAnimation';
import { useLottieAnimation } from './useLottieAnimation';

type Feature = {
  img: string;
  url: string;
  title: string;
  description: string;
  lottieJsonUrl: any;
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
          <img
            className={styles.whyRspackBg}
            src="https://assets.rspack.dev/rspack/assets/landingpage-why-rspack-card-why-bg.png"
            alt="bg"
          />
        </div>
      </div>
    </div>
  );
};

const FeatureItem = memo(({ feature }: { feature: Feature }) => {
  const { description, img, title, url, lottieJsonUrl } = feature;
  const {
    container,
    isHovering,
    onMouseEnter,
    onMouseLeave,
    onMouseMove,
    onTouchEnd,
    onTouchMove,
    onTouchStart,
    outerContainer,
    ref: cardAnimationContainerRef,
    shine,
    shineBg,
  } = useCardAnimation();

  const { ref: lottieContainerRef } = useLottieAnimation(
    isHovering,
    lottieJsonUrl,
  );

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
      ref={cardAnimationContainerRef as any}
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
          <img
            src={img}
            alt={title}
            className={styles.featureIconImg}
            style={{
              display: isHovering ? 'none' : 'flex',
            }}
          />
          <div
            ref={lottieContainerRef as any}
            className={styles.featureIconImg}
            style={{ display: isHovering ? 'flex' : 'none' }}
          />
        </div>
        <div className={styles.featureContent}>
          <h3 className={styles.featureTitle}>{title}</h3>
          <p className={styles.featureDescription}>{description}</p>
        </div>
      </Link>
    </div>
  );
});

const WhyRspack = memo(() => {
  const t = useI18n();
  const tUrl = useI18nUrl();

  const features: Feature[] = useMemo(
    () => [
      {
        img: Speedometer,
        url: tUrl('/guide/start/introduction'),
        title: t('FastStartup'),
        description: t('FastStartupDesc'),
        lottieJsonUrl: SpeedometerJson,
      },
      {
        img: Lightning,
        url: tUrl('/guide/start/introduction'),
        title: t('LightningHMR'),
        description: t('LightningHMRDesc'),
        lottieJsonUrl: LightningJson,
      },
      {
        img: FrameCheck,
        url: tUrl('/guide/tech/react'),
        title: t('FrameworkAgnostic'),
        description: t('FrameworkAgnosticDesc'),
        lottieJsonUrl: FrameCheckJson,
      },
      {
        img: Compatible,
        url: tUrl('/guide/compatibility/plugin'),
        title: t('WebpackCompatible'),
        description: t('WebpackCompatibleDesc'),
        lottieJsonUrl: CompatibleJson,
      },
    ],
    [t, tUrl],
  );

  return (
    <section className={sharedStyles.container}>
      <div className={sharedStyles.innerContainer}>
        <div className={styles.features}>
          {/* Why Rspack? */}
          <WhyRspackCard />
          {features.map(feature => {
            return <FeatureItem key={feature.title} feature={feature} />;
          })}
        </div>
      </div>
    </section>
  );
});

export default WhyRspack;
