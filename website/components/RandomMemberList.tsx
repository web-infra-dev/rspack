import { NoSSR } from '@rspress/core/runtime';
import { Link } from '@rspress/core/theme';
import style from './RandomMemberList.module.scss';

interface Member {
  id: string;
  avatar: string;
  // The display name, if not set, use id instead
  name?: string;
  desc?: string;
  x?: string;
  bluesky?: string;
}

const TwitterSVG = (
  <svg
    role="img"
    viewBox="0 0 24 24"
    width="24"
    height="24"
    xmlns="http://www.w3.org/2000/svg"
  >
    <title>X</title>
    <path d="M18.901 1.153h3.68l-8.04 9.19L24 22.846h-7.406l-5.8-7.584-6.638 7.584H.474l8.6-9.83L0 1.154h7.594l5.243 6.932ZM17.61 20.644h2.039L6.486 3.24H4.298Z" />
  </svg>
);

const GitHubSVG = (
  <svg
    role="img"
    viewBox="0 0 24 24"
    width="24"
    height="24"
    xmlns="http://www.w3.org/2000/svg"
  >
    <title>GitHub</title>
    <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
  </svg>
);

const BlueskySVG = (
  <svg xmlns="http://www.w3.org/2000/svg" width="100%" viewBox="0 0 24 24">
    <title>Bluesky</title>
    <path d="M12 10.8c-1.087-2.114-4.046-6.053-6.798-7.995C2.566.944 1.561 1.266.902 1.565C.139 1.908 0 3.08 0 3.768c0 .69.378 5.65.624 6.479c.815 2.736 3.713 3.66 6.383 3.364q.204-.03.415-.056q-.207.033-.415.056c-3.912.58-7.387 2.005-2.83 7.078c5.013 5.19 6.87-1.113 7.823-4.308c.953 3.195 2.05 9.271 7.733 4.308c4.267-4.308 1.172-6.498-2.74-7.078a9 9 0 0 1-.415-.056q.21.026.415.056c2.67.297 5.568-.628 6.383-3.364c.246-.828.624-5.79.624-6.478c0-.69-.139-1.861-.902-2.206c-.659-.298-1.664-.62-4.3 1.24C16.046 4.748 13.087 8.687 12 10.8" />
  </svg>
);

const coreTeam: Member[] = [
  {
    id: 'zoolsher',
    avatar: 'https://github.com/zoolsher.png',
    x: 'https://x.com/zoolsher',
    desc: 'Rspack core team',
  },
  {
    id: 'hardfist',
    avatar: 'https://github.com/hardfist.png',
    x: 'https://x.com/hardfist_1',
    bluesky: 'https://bsky.app/profile/hardfist1.bsky.social',
    desc: 'Rspack core team',
  },
  {
    id: 'jkzing',
    desc: 'Rspack core team, Vue contributor',
    x: 'https://x.com/zjkdddd',
    avatar: 'https://github.com/jkzing.png',
  },
  {
    id: 'ahabhgk',
    avatar: 'https://github.com/ahabhgk.png',
    x: 'https://x.com/ahabhgk',
    bluesky: 'https://bsky.app/profile/ahabhgk.bsky.social',
    desc: 'Rspack core team, webpack contributor',
  },
  {
    id: 'jerrykingxyz',
    avatar: 'https://github.com/jerrykingxyz.png',
    desc: 'Rspack core team',
  },
  {
    id: 'chenjiahan',
    avatar: 'https://github.com/chenjiahan.png',
    x: 'https://x.com/jait_chen',
    bluesky: 'https://bsky.app/profile/chenjiahan.bsky.social',
    desc: 'Rspack core team, project lead of Vant',
  },
  {
    id: 'JSerFeng',
    avatar: 'https://github.com/JSerFeng.png',
    x: 'https://x.com/JSerFeng',
    bluesky: 'https://bsky.app/profile/jserfeng.bsky.social',
    desc: 'Rspack core team',
  },
  {
    id: '9aoy',
    avatar: 'https://github.com/9aoy.png',
    desc: 'Rspack core team',
  },
  {
    id: 'zackarychapple',
    avatar: 'https://github.com/zackarychapple.png',
    desc: 'Rspack core team, CEO at ZephyrCloudIO',
    x: 'https://x.com/Zackary_Chapple',
  },
  {
    id: 'valorkin',
    avatar: 'https://github.com/valorkin.png',
    desc: 'Rspack core team, CTO at ZephyrCloudIO',
    x: 'https://x.com/valorkin',
  },
  {
    id: 'lingyucoder',
    avatar: 'https://github.com/lingyucoder.png',
    x: 'https://x.com/lingyucoder',
    desc: 'Rspack core team',
  },
  {
    id: 'ScriptedAlchemy',
    avatar: 'https://github.com/ScriptedAlchemy.png',
    desc: 'Inventor of Module Federation, Rspack & webpack core team',
    bluesky: 'https://bsky.app/profile/scriptedalchemy.bsky.social',
    x: 'https://x.com/ScriptedAlchemy',
  },
  {
    id: 'SyMind',
    avatar: 'https://github.com/SyMind.png',
    desc: 'Rspack core team',
  },
  {
    id: 'xc2',
    avatar: 'https://github.com/xc2.png',
    x: 'https://x.com/kfll',
    desc: 'Rspack core team',
  },
  {
    id: 'fi3ework',
    avatar: 'https://github.com/fi3ework.png',
    x: 'https://x.com/f13wk',
    desc: 'Rspack core team, creator of vite-plugin-checker, webpack contributor',
  },
  {
    id: 'easy1090',
    avatar: 'https://github.com/easy1090.png',
    x: 'https://x.com/yifan56737904',
    desc: 'Rspack core team',
  },
  {
    id: 'Timeless0911',
    avatar: 'https://github.com/Timeless0911.png',
    x: 'https://x.com/',
    desc: 'Rspack core team',
  },
  {
    id: 'SoonIter',
    avatar: 'https://github.com/SoonIter.png',
    x: 'https://x.com/Soon_Iter',
    bluesky: 'https://bsky.app/profile/sooniter.bsky.social',
    desc: 'Rspack core team',
  },
  {
    id: 'inottn',
    avatar: 'https://github.com/inottn.png',
    x: 'https://x.com/inorr_r',
    desc: 'Rspack & Vant core team',
  },
  {
    id: 'GiveMe-A-Name',
    avatar: 'https://github.com/GiveMe-A-Name.png',
    x: 'https://x.com/qixuan_xie',
    desc: 'Rspack core team, Modern.js contributor',
  },
  {
    id: 'nyqykk',
    avatar: 'https://github.com/nyqykk.png',
    desc: 'Rspack core team, Module Federation contributor',
  },
  {
    id: 'stormslowly',
    avatar: 'https://github.com/stormslowly.png',
    desc: 'Rspack core team',
  },
  {
    id: 'CPunisher',
    avatar: 'https://github.com/CPunisher.png',
    x: 'https://x.com/c_punisher',
    desc: 'Rspack & SWC core team',
  },
  {
    id: 'quininer',
    avatar: 'https://github.com/quininer.png',
    desc: 'Rspack core team, creator of io-uring',
    x: 'https://x.com/quininers',
  },
  {
    id: 'fansenze',
    avatar: 'https://github.com/fansenze.png',
    desc: 'Rspack core team',
  },
];

export const RandomMemberList = ({ list = coreTeam }: { list?: Member[] }) => {
  const randomList = list.sort(() => Math.random() - 0.5);
  return (
    <NoSSR>
      <div className={`${style.wrapper} rp-not-doc`}>
        {randomList.map((item) => (
          <div className={style.card} key={item.id}>
            <img className={style.avatar} src={item.avatar} alt="avatar" />
            <div className={style.name}>{item.name || item.id}</div>
            <div className={style.desc}>{item.desc}</div>
            <div className={style.icons}>
              <Link
                className={style.icon}
                href={`https://github.com/${item.id}`}
              >
                {GitHubSVG}
              </Link>
              {item.x ? (
                <Link className={style.icon} href={item.x}>
                  {TwitterSVG}
                </Link>
              ) : null}
              {item.bluesky ? (
                <Link className={style.icon} href={item.bluesky}>
                  {BlueskySVG}
                </Link>
              ) : null}
            </div>
          </div>
        ))}
      </div>
    </NoSSR>
  );
};

export const RandomContributorsList = () => {
  const list: Member[] = [
    {
      id: 'hyf0',
      avatar: 'https://github.com/hyf0.png',
      x: 'https://x.com/_hyf0',
      desc: 'Rspack & Rolldown core team',
    },
    {
      id: 'underfin',
      avatar: 'https://github.com/underfin.png',
      desc: 'Rspack & Rolldown core team, Vite contributor',
    },
    {
      id: 'Boshen',
      avatar: 'https://github.com/Boshen.png',
      x: 'https://x.com/boshen_c',
      desc: 'Rspack & Rolldown core team, creator of Oxc',
    },
    {
      id: 'IWANABETHATGUY',
      avatar: 'https://github.com/IWANABETHATGUY.png',
      desc: 'Rspack & Rolldown core team',
    },
    {
      id: 'suxin2017',
      avatar: 'https://github.com/suxin2017.png',
      x: 'https://x.com/suxin2017',
      desc: 'Rspack core team, Biome contributor',
    },
    {
      id: 'sanyuan0704',
      avatar: 'https://github.com/sanyuan0704.png',
      x: 'https://x.com/sanyuan0704',
      desc: 'Rspack core team, creator of Rspress',
    },
    {
      id: 'shulaoda',
      avatar: 'https://github.com/shulaoda.png',
      x: 'https://x.com/dalaoshv',
      desc: 'Rspack & Rolldown core team',
    },
    {
      id: 'bvanjoi',
      avatar: 'https://github.com/bvanjoi.png',
      desc: 'Rspack & SWC core team, Rust language contributor',
    },
    {
      id: 'h-a-n-a',
      name: 'Hana',
      avatar: 'https://github.com/h-a-n-a.png',
      x: 'https://x.com/_h_ana___',
      desc: 'Rspack & Rolldown core team',
    },
  ];

  return <RandomMemberList list={list} />;
};
