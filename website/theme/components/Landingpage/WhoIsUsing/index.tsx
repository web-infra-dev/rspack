import { BuiltWithRspack } from '@rstack-dev/doc-ui/built-with-rspack';
import {
  containerStyle,
  innerContainerStyle,
} from '@rstack-dev/doc-ui/section-style';
import { memo } from 'react';
import { Link } from 'rspress/theme';
import { useI18n } from '../../../i18n';
import abbLogo from './assets/abb.svg';
import alibabaLogo from './assets/alibaba.svg';
import amazonLogo from './assets/amazon.svg';
import bitDevLogo from './assets/bit.svg';
import bytedanceLogo from './assets/bytedance.svg';
import deepseekLogo from './assets/deepseek.svg';
import discordLogo from './assets/discord.svg';
import getaroundLogo from './assets/getaround.svg';
import googleLogo from './assets/google.svg';
import intuitLogo from './assets/intuit.svg';
import khanAcademyLogo from './assets/khan-academy.svg';
import kuaishouLogo from './assets/kuaishou.svg';
import microsoftLogo from './assets/microsoft.svg';
import nioLogo from './assets/nio.svg';
import sequoiaLogo from './assets/sequoia.svg';
import tiktokLogo from './assets/tiktok.svg';
import trellisLogo from './assets/trellis.svg';
import verkadaLogo from './assets/verkada.png';

type Company = {
  name: string;
  logo: string;
  url: string;
  text?: string;
  width?: string | number;
};

const companyList: Company[] = [
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
    width: 110,
  },
  {
    name: 'ByteDance',
    logo: bytedanceLogo,
    url: 'https://www.bytedance.com',
    width: 180,
  },
  {
    name: 'TikTok',
    logo: tiktokLogo,
    url: 'https://www.tiktok.com',
    width: 180,
  },
  {
    name: 'Google',
    logo: googleLogo,
    url: 'https://google.com',
    width: 110,
  },
  {
    name: 'Alibaba',
    logo: alibabaLogo,
    url: 'https://www.alibaba.com',
    width: 160,
  },
  {
    name: 'bit.dev',
    logo: bitDevLogo,
    text: 'bit.dev',
    url: 'https://bit.dev/',
    width: 40,
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
    width: 110,
  },
  {
    name: 'ABB',
    logo: abbLogo,
    url: 'https://abb-bank.az/en',
    width: 100,
  },
  {
    name: 'Sequoia',
    logo: sequoiaLogo,
    url: 'https://www.sequoia.com/',
    width: 150,
  },
  {
    name: 'Getaround',
    logo: getaroundLogo,
    url: 'https://getaround.com',
    width: 130,
  },
  {
    name: 'Trellis',
    logo: trellisLogo,
    url: 'https://trellis.org',
    width: 100,
  },
  {
    name: 'Kuaishou',
    logo: kuaishouLogo,
    url: 'https://ir.kuaishou.com/',
    width: 160,
  },
  {
    name: 'DeepSeek',
    logo: deepseekLogo,
    url: 'https://deepseek.com',
    width: 140,
  },
  {
    name: 'Khan Academy',
    logo: khanAcademyLogo,
    url: 'https://khanacademy.org/',
    width: 180,
  },
  {
    name: 'Verkada',
    logo: verkadaLogo,
    url: 'https://verkada.com',
    width: 180,
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
