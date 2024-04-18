import { NoSSR } from 'rspress/runtime';
import style from './RandomMemberList.module.scss';

interface Member {
  id: string;
  avatar: string;
  // The display name, if not set, use id instead
  name?: string;
}

export const RandomMemberList = () => {
  const list: Member[] = [
    {
      id: 'zoolsher',
      avatar: 'https://avatars.githubusercontent.com/u/9161085?s=120&v=4',
    },
    {
      id: 'hardfist',
      avatar: 'https://avatars.githubusercontent.com/u/8898718?s=120&v=4',
    },
    {
      id: 'ahabhgk',
      avatar: 'https://avatars.githubusercontent.com/u/42857895?s=120&v=4',
    },
    {
      id: 'bvanjoi',
      avatar: 'https://avatars.githubusercontent.com/u/30187863?s=120&v=4',
    },
    {
      id: 'Boshen',
      avatar: 'https://avatars.githubusercontent.com/u/1430279?s=120&v=4',
    },
    {
      id: 'h-a-n-a',
      name: 'Hana',
      avatar: 'https://avatars.githubusercontent.com/u/10465670?s=120&v=4',
    },

    {
      id: 'IWANABETHATGUY',
      avatar: 'https://avatars.githubusercontent.com/u/17974631?s=120&v=4',
    },
    {
      id: 'jerrykingxyz',
      avatar: 'https://avatars.githubusercontent.com/u/9291503?s=120&v=4',
    },

    {
      id: 'chenjiahan',
      avatar: 'https://avatars.githubusercontent.com/u/7237365?s=120&v=4',
    },
    {
      id: 'JSerFeng',
      avatar: 'https://avatars.githubusercontent.com/u/57202839?s=120&v=4',
    },
    {
      id: '9aoy',
      avatar: 'https://avatars.githubusercontent.com/u/22373761?s=120&v=4',
    },
    {
      id: 'sanyuan0704',
      avatar: 'https://avatars.githubusercontent.com/u/39261479?s=120&v=4',
    },
    {
      id: 'suxin2017',
      avatar: 'https://avatars.githubusercontent.com/u/28481035?v=4',
    },
    {
      id: 'zackarychapple',
      avatar: 'https://avatars.githubusercontent.com/u/2133184?v=4',
    },
    {
      id: 'valorkin',
      avatar: 'https://avatars.githubusercontent.com/u/1107171?v=4',
    },
    {
      id: 'lingyucoder',
      avatar: 'https://avatars.githubusercontent.com/u/2663351?v=4',
    },
    {
      id: 'ScriptedAlchemy',
      avatar: 'https://avatars.githubusercontent.com/u/25274700?v=4',
    },
    {
      id: 'SyMind',
      avatar: 'https://avatars.githubusercontent.com/u/19852293?v=4',
    },
    {
      id: 'xc2',
      avatar: 'https://avatars.githubusercontent.com/u/18117084?v=4',
    },
  ];

  const randomList = list.sort(() => Math.random() - 0.5);

  return (
    <NoSSR>
      <div className={style.wrapper}>
        {randomList.map(item => (
          <a
            className={style.link}
            href={`https://github.com/${item.id}`}
            target="_blank"
            rel="nofollow"
            key={item.id}
            style={{
              border: 'none',
            }}
          >
            <img className={style.avatar} src={item.avatar} />
            <span className={style.name}>{item.name || item.id}</span>
          </a>
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
    },
    {
      id: 'underfin',
      avatar: 'https://avatars.githubusercontent.com/u/14008915?s=120&v=4',
    },
  ];
  const randomList = list.sort(() => Math.random() - 0.5);
  return (
    <NoSSR>
      <div className={style.wrapper}>
        {randomList.map(item => (
          <a
            className={style.link}
            href={`https://github.com/${item.id}`}
            target="_blank"
            rel="nofollow"
            key={item.id}
            style={{
              border: 'none',
            }}
          >
            <img className={style.avatar} src={item.avatar} />
            <span className={style.name}>{item.name || item.id}</span>
          </a>
        ))}
      </div>
    </NoSSR>
  );
};
