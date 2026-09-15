import { useLang } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import type { CSSProperties, FC } from 'react';

const LICENSE_URL = 'https://creativecommons.org/licenses/by/4.0/';

const rootStyle: CSSProperties = {
  borderTop: '1px solid var(--rp-c-divider-light)',
  color: 'var(--rp-c-text-2)',
  fontSize: '13px',
  marginTop: '48px',
  paddingTop: '16px',
};

const Attribution: FC<{ url: string }> = ({ url }) => {
  const isEn = useLang() === 'en';

  if (isEn) {
    return (
      <p style={rootStyle}>
        This page is adapted from <Link href={url}>webpack documentation</Link>{' '}
        under the <Link href={LICENSE_URL}>CC BY 4.0</Link>, with modifications.
      </p>
    );
  }

  return (
    <p style={rootStyle}>
      本页改编自 <Link href={url}>webpack 文档</Link>，遵循{' '}
      <Link href={LICENSE_URL}>CC BY 4.0</Link>，且已作修改。
    </p>
  );
};

export default Attribution;
