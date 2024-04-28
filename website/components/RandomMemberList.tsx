import { NoSSR } from 'rspress/runtime';
import style from './RandomMemberList.module.scss';

interface Member {
  id: string;
  avatar: string;
  // The display name, if not set, use id instead
  name?: string;
  desc?: string;
  x?: string;
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

const coreTeam: Member[] = [
  {
    id: 'zoolsher',
    avatar: 'https://avatars.githubusercontent.com/u/9161085?s=120&v=4',
    x: 'https://twitter.com/zoolsher',
    desc: 'Rspack contributor',
  },
  {
    id: 'hardfist',
    avatar: 'https://avatars.githubusercontent.com/u/8898718?s=120&v=4',
    x: 'https://twitter.com/hardfist_1',
    desc: 'Rspack contributor',
  },
  {
    id: 'jkzing',
    desc: 'Rspack contributor',
    avatar: 'https://avatars.githubusercontent.com/u/2851517?v=4',
  },
  {
    id: 'ahabhgk',
    avatar: 'https://avatars.githubusercontent.com/u/42857895?s=120&v=4',
    x: 'https://twitter.com/ahabhgk',
    desc: 'Rspack / Webpack contributor',
  },
  {
    id: 'bvanjoi',
    avatar: 'https://avatars.githubusercontent.com/u/30187863?s=120&v=4',
    desc: 'Rspack / Rust contributor',
  },
  {
    id: 'Boshen',
    avatar: 'https://avatars.githubusercontent.com/u/1430279?s=120&v=4',
    x: 'https://twitter.com/boshen_c',
    desc: 'Rspack / Oxc contributor',
  },
  {
    id: 'h-a-n-a',
    name: 'Hana',
    avatar: 'https://avatars.githubusercontent.com/u/10465670?s=120&v=4',
    x: 'https://twitter.com/_h_ana___',
    desc: 'Rspack contributor',
  },
  {
    id: 'IWANABETHATGUY',
    avatar: 'https://avatars.githubusercontent.com/u/17974631?s=120&v=4',
    desc: 'Rspack contributor',
  },
  {
    id: 'jerrykingxyz',
    avatar: 'https://avatars.githubusercontent.com/u/9291503?s=120&v=4',
    desc: 'Rspack contributor',
  },
  {
    id: 'chenjiahan',
    avatar: 'https://avatars.githubusercontent.com/u/7237365?s=120&v=4',
    x: 'https://twitter.com/jait_chen',
    desc: 'Rspack / Rsbuild / Vant contributor',
  },
  {
    id: 'JSerFeng',
    avatar: 'https://avatars.githubusercontent.com/u/57202839?s=120&v=4',
    x: 'https://twitter.com/JSerFeng',
    desc: 'Rspack contributor',
  },
  {
    id: '9aoy',
    avatar: 'https://avatars.githubusercontent.com/u/22373761?s=120&v=4',
    desc: 'Rspack / Rsbuild contributor',
  },
  {
    id: 'sanyuan0704',
    avatar: 'https://avatars.githubusercontent.com/u/39261479?s=120&v=4',
    x: 'https://twitter.com/sanyuan0704',
    desc: 'Rspack / Rspress contributor',
  },
  {
    id: 'suxin2017',
    avatar: 'https://avatars.githubusercontent.com/u/28481035?v=4',
    x: 'https://twitter.com/suxin2017',
    desc: 'Rspack contributor',
  },
  {
    id: 'zackarychapple',
    avatar: 'https://avatars.githubusercontent.com/u/2133184?v=4',
    desc: 'Rspack contributor, CEO at ZephyrCloudIO',
  },
  {
    id: 'valorkin',
    avatar: 'https://avatars.githubusercontent.com/u/1107171?v=4',
    desc: 'Rspack contributor, CTO at ZephyrCloudIO',
    x: 'https://twitter.com/valorkin',
  },
  {
    id: 'lingyucoder',
    avatar: 'https://avatars.githubusercontent.com/u/2663351?v=4',
    x: 'https://twitter.com/lingyucoder',
    desc: 'Rspack contributor',
  },
  {
    id: 'ScriptedAlchemy',
    avatar: 'https://avatars.githubusercontent.com/u/25274700?v=4',
    desc: 'Inventor of Module Federation, Rspack / Webpack contributor',
  },
  {
    id: 'SyMind',
    avatar: 'https://avatars.githubusercontent.com/u/19852293?v=4',
    desc: 'Rspack contributor',
  },
  {
    id: 'xc2',
    avatar: 'https://avatars.githubusercontent.com/u/18117084?v=4',
    x: 'https://twitter.com/kfll',
    desc: 'Rspack contributor',
  },
];

export const RandomMemberList = ({ list = coreTeam }: { list: Member[] }) => {
  const randomList = list.sort(() => Math.random() - 0.5);
  return (
    <NoSSR>
      <div className={style.wrapper}>
        {randomList.map(item => (
          <div className={style.card} key={item.id}>
            <img className={style.avatar} src={item.avatar} alt="avatar" />
            <div className={style.name}>{item.name || item.id}</div>
            <div className={style.desc}>{item.desc}</div>
            <div className={style.icons}>
              <a
                className={style.icon}
                href={`https://github.com/${item.id}`}
                target="_blank"
                rel="noreferrer"
              >
                {GitHubSVG}
              </a>
              {item.x ? (
                <a
                  className={style.icon}
                  href={`https://github.com/${item.id}`}
                  target="_blank"
                  rel="noreferrer"
                >
                  {TwitterSVG}
                </a>
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
      avatar: 'https://avatars.githubusercontent.com/u/49502170?s=120&v=4',
      x: 'https://twitter.com/_hyf0',
      desc: 'Rspack / Rolldown contributor',
    },
    {
      id: 'underfin',
      avatar: 'https://avatars.githubusercontent.com/u/14008915?s=120&v=4',
      desc: 'Rspack / Rolldown / Vite contributor',
    },
  ];

  const randomList = list.sort(() => Math.random() - 0.5);
  return <RandomMemberList list={randomList} />;
};
