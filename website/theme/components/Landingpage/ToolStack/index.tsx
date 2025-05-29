import { containerStyle } from '@rstack-dev/doc-ui/section-style';
import { ToolStack as BaseToolStack } from '@rstack-dev/doc-ui/tool-stack';
import type React from 'react';
import { memo } from 'react';
import { useLang } from 'rspress/runtime';
import { useI18n } from '../../../i18n';

const ToolStack: React.FC = memo(() => {
  const lang = useLang();
  const t = useI18n();

  return (
    <section className={containerStyle}>
      <BaseToolStack lang={lang} />
    </section>
  );
});

export default ToolStack;
