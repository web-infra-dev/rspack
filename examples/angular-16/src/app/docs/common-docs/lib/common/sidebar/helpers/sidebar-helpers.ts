import { Route, Routes } from "@angular/router";
import { SidebarRoutesType, NestedRouteType, SidebarRoutesStructure } from '../../../models/sidebar-routes.model';

export function initNestedRoutes(routes: Routes, sideBarMenu: SidebarRoutesType): SidebarRoutesType {
  routes.forEach(item => {
    if (item.children?.length) {
      item.children.forEach(childRoute => {
        const key = childRoute.data?.[1]?.sideBarParentTitle;
        initSideBarItem(key, childRoute, sideBarMenu);
      });
    }

    if (!item.children?.length) {
      const key = item.data?.[1]?.sideBarParentTitle;
      initSideBarItem(key, item, sideBarMenu);
    }
  });
  return sideBarMenu;
}

function initSideBarItem(key: string, route: Route, sideBarMenu: SidebarRoutesType) {
  if (key && sideBarMenu[key as keyof SidebarRoutesType]) {
    const sideBarItem = sideBarMenu[key as keyof SidebarRoutesType];
    const nestedItem: NestedRouteType = {
      title: route.data?.[0],
      path: route.data?.[1]?.parentRoute ? `/${route.data?.[1]?.parentRoute}/${route.path}` : route.path,
      isOpened: false,
      fragments: key === 'components' ? initFragments() : []
    };

    if (!SidebarRoutesStructure[key as keyof SidebarRoutesType]?.nestedRoutes.filter( menuItem => menuItem.title === nestedItem.title).length) {
      sideBarItem.nestedRoutes.push(nestedItem);
    }
  }
}

function initFragments(): {title: string, path: string, isOpened: boolean}[] {
  return [
    {
      title: 'Overview',
      path: 'overview',
      isOpened: false
    },
    {
      title: 'API',
      path: 'api',
      isOpened: false
    },
    {
      title: 'Examples',
      path: 'examples',
      isOpened: false
    }
  ];
}
