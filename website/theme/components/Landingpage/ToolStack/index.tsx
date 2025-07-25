import { useLang } from '@rspress/core/runtime';
import { containerStyle } from '@rstack-dev/doc-ui/section-style';
import { ToolStack as BaseToolStack } from '@rstack-dev/doc-ui/tool-stack';
import type React from 'react';
import { memo } from 'react';

const ToolStack: React.FC = memo(() => {
  const lang = useLang();

  return (
    <section className={containerStyle}>
      <BaseToolStack lang={lang} />
    </section>
  );
});

export default ToolStack;
