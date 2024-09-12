import { BuiltWithRspack } from '@rstack-dev/doc-ui/built-with-rspack';
import {
  containerStyle,
  innerContainerStyle,
} from '@rstack-dev/doc-ui/section-style';
import { memo } from 'react';
import { Link } from 'rspress/theme';
import { useI18n } from '../../../i18n';
import amazonLogo from './assets/amazon.svg';
import bitDevLogo from './assets/bit.svg';
import bytedanceLogo from './assets/bytedance.svg';
import discordLogo from './assets/discord.svg';
import getaroundLogo from './assets/getaround.svg';
import intuitLogo from './assets/intuit.svg';
import microsoftLogo from './assets/microsoft.svg';
import nioLogo from './assets/nio.svg';

type Company = {
  name: string;
  logo: string;
  url: string;
  text?: string;
  width?: string | number;
};

const companyList: Company[] = [
  {
    name: 'bit.dev',
    logo: bitDevLogo,
    text: 'bit.dev',
    url: 'https://bit.dev/',
    width: 40,
  },
  {
    name: 'Microsoft',
    logo: microsoftLogo,
    url: 'https://www.microsoft.com',
    width: 180,
  },
  {
    name: 'Amazon',
    logo: amazonLogo,
    url: 'https://amazon.com/',
    width: 120,
  },
  {
    name: 'ByteDance',
    logo: bytedanceLogo,
    url: 'https://www.bytedance.com',
    width: 180,
  },
  {
    name: 'Intuit',
    logo: intuitLogo,
    url: 'https://www.intuit.com',
    width: 100,
  },
  {
    name: 'Discord',
    logo: discordLogo,
    url: 'https://discord.com',
    width: 140,
  },
  {
    name: 'NIO',
    logo: nioLogo,
    url: 'https://nio.com',
    width: 115,
  },
  {
    name: 'Getaround',
    logo: getaroundLogo,
    url: 'https://getaround.com',
    width: 110,
  },
];

const WhoIsUsing: React.FC = memo(() => {
  const t = useI18n();
  return (
    <section className={containerStyle}>
      <div className={innerContainerStyle}>
        <BuiltWithRspack
          companyList={companyList}
          title={t('whoIsUsing')}
          LinkComp={Link}
        />
      </div>
    </section>
  );
});

export default WhoIsUsing;
