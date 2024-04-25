import { Link } from 'rspress/theme';
import { useLang } from 'rspress/runtime';
import { useI18n } from '../../i18n/index';

function useFooterData() {
  const t = useI18n();
  const lang = useLang();
  const getLink = (link: string) => (lang === 'en' ? link : `/${lang}${link}`);

  return [
    {
      title: t('guide'),
      items: [
        {
          title: t('quickStart'),
          link: getLink('/guide/start/quick-start'),
        },
        {
          title: t('features'),
          link: getLink('/guide/features/asset-module'),
        },
        {
          title: t('migration'),
          link: getLink('/guide/migration/webpack'),
        },
        {
          title: t('compatibility'),
          link: getLink('/guide/compatibility/loader'),
        },
      ],
    },
    {
      title: 'API',
      items: [
        {
          title: t('cli'),
          link: getLink('/api/cli'),
        },
        {
          title: 'Plugin API',
          link: getLink('/api/plugin-api'),
        },
        {
          title: 'Loader API',
          link: getLink('/api/loader-api'),
        },
      ],
    },
    {
      title: t('ecosystem'),
      items: [
        {
          title: 'Rsbuild',
          link: 'https://rsbuild.dev/',
        },
        {
          title: 'Rspress',
          link: 'https://rspress.dev/',
        },
        {
          title: 'Rsdoctor',
          link: 'https://rsdoctor.dev/',
        },
        {
          title: 'Modern.js',
          link: 'https://modernjs.dev/en/',
        },
      ],
    },
    {
      title: t('community'),
      items: [
        {
          title: 'GitHub',
          link: 'https://github.com/web-infra-dev/rspack',
        },
        {
          title: 'Discord',
          link: 'https://discord.gg/ab2Rv4BXwf',
        },
        {
          title: 'X',
          link: 'https://twitter.com/rspack_dev',
        },
      ],
    },
  ];
}

export function HomeFooter() {
  const footerData = useFooterData();
  return (
    <div
      className="flex flex-col border-t items-center mt-12"
      style={{ borderColor: 'var(--rp-c-divider-light)' }}
    >
      <div className="pt-8 pb-4 w-full justify-around max-w-6xl hidden sm:flex">
        {footerData.map(item => (
          <div key={item.title} className="flex flex-col items-start">
            <h2 className="font-bold my-4 text-lg">{item.title}</h2>
            <ul className="flex flex-col gap-3">
              {item.items.map(subItem => (
                <li key={subItem.title}>
                  <Link href={subItem.link}>
                    <span className="font-normal">{subItem.title}</span>
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="flex flex-center">
        <h2 className="font-normal text-sm text-gray-600 dark:text-light-600 py-4">
          © 2022-present ByteDance Inc. All Rights Reserved.
        </h2>
      </div>
    </div>
  );
}
