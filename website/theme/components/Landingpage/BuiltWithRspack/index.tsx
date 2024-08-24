import { Link } from 'rspress/theme';
import { useI18n } from '../../../i18n';
import sharedStyles from '../shared.module.scss';
import amazonLogo from './assets/amazon.svg';
import bitDevLogo from './assets/bit.svg';
import bytedanceLogo from './assets/bytedance.svg';
import discordLogo from './assets/discord.svg';
import intuitLogo from './assets/intuit.svg';
import microsoftLogo from './assets/microsoft.svg';
import styles from './index.module.scss';

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
];

const CompanyItem = ({ item }: { item: Company }) => {
  const { logo, name, url, text, width } = item;
  return (
    <Link className={styles.logo} href={url}>
      <img src={logo} alt={name} style={{ width }} />
      {text !== undefined ? (
        <span className={styles.logoText}>{text}</span>
      ) : (
        <></>
      )}
    </Link>
  );
};

const BuiltWithRsPack: React.FC = () => {
  const t = useI18n();
  return (
    <section className={sharedStyles.container}>
      <div
        className={`${sharedStyles.innerContainer} ${styles.innerContainer}`}
      >
        <style>
          {`:root {
            --landingpage-built-with-rspack-logo-size: 1;
            } `}
        </style>
        <h2 className={styles.title}>{t('builtWithRspack')}</h2>
        <div className={styles.logos}>
          {companyList.map(i => {
            return <CompanyItem key={i.name} item={i} />;
          })}
        </div>
      </div>
    </section>
  );
};

export default BuiltWithRsPack;
