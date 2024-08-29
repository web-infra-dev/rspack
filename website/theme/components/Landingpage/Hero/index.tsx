import { Hero as BaseHero } from '@rstack-dev/doc-ui/hero';
import { memo, useCallback } from 'react';
import { useNavigate } from 'rspress/runtime';
import { useI18n, useI18nUrl } from '../../../i18n';

const Hero = memo(() => {
  const tUrl = useI18nUrl();
  const t = useI18n();

  const navigate = useNavigate();
  const handleClickGetStarted = useCallback(() => {
    navigate(tUrl('/guide/start/quick-start'));
  }, [tUrl, navigate]);

  const handleClickLearnMore = useCallback(() => {
    navigate(tUrl('/guide/start/introduction'));
  }, [tUrl, navigate]);

  return (
    <BaseHero
      showStars
      title="Rspack"
      subTitle={t('heroSlogan')}
      description={t('heroSubSlogan')}
      getStartedButtonText={t('getStarted')}
      learnMoreButtonText={t('learnMore')}
      onClickGetStarted={handleClickGetStarted}
      onClickLearnMore={handleClickLearnMore}
    />
  );
});

export default Hero;
