export type SidebarRoutesType = {
  [key in "documentation" | "components" | "themes"]: SidebarRouteItemValueType;
};

export type SidebarRouteItemValueType = {
  nestedRoutes: NestedRouteType[];
  isOpened: boolean;
  title: string;
  icon: string;
  path?: string;
};

export type NestedRouteType = {
  title: string;
  path?: string;
  isOpened: boolean;
  fragments: {
    title: string;
    path: string;
    isOpened: boolean;
  }[] | [];
};

export const SidebarRoutesStructure: Partial<SidebarRoutesType> = {
  documentation: {
    nestedRoutes:[],
    isOpened: false,
    title: 'DOCUMENTATION',
    icon: 'assets/images/icons/icon-folder.svg',
    path: 'documentation'
  },
  components: {
    nestedRoutes: [],
    isOpened: false,
    title: 'COMPONENTS',
    icon: 'assets/images/icons/icon-components.svg',
    path: 'components'
  }
  // themes: {
  //   nestedRoutes:[],
  //   isOpened: false,
  //   title: 'THEMES',
  //   icon: 'assets/images/icons/icon-theme.svg',
  //   path: 'themes'
  // }
};
