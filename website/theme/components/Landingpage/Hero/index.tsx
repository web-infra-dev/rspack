import { Hero as BaseHero } from '@rstack-dev/doc-ui/hero';
import { memo, useCallback } from 'react';
import { useNavigate } from 'rspress/runtime';
import { useI18n, useI18nUrl } from '../../../i18n';

const Hero = memo(() => {
  const tUrl = useI18nUrl();
  const t = useI18n();

  const navigate = useNavigate();
  const onClickGetStarted = useCallback(() => {
    navigate(tUrl('/guide/start/quick-start'));
  }, [tUrl, navigate]);

  return (
    <BaseHero
      showStars
      title="Rspack"
      subTitle={t('heroSlogan')}
      description={t('heroSubSlogan')}
      getStartedButtonText={t('getStarted')}
      githubURL="https://github.com/web-infra-dev/rspack"
      onClickGetStarted={onClickGetStarted}
    />
  );
});

export default Hero;
