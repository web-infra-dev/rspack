import { Routes } from '@angular/router';
import {
  LandingComponent,
  ComponentsPageComponent,
  DocumentationComponent,
  DiscoverComponent,
  SchematicsComponent,
  ConstructionPageComponent
} from "./docs/common-docs";

export const routes: Routes = [
  {
    path: '',
    data: ['Landing page'],
    component: LandingComponent
  },
  {
    path: 'documentation',
    data: ['Documentation', {sideBarParentTitle: 'documentation'}],
    component: DocumentationComponent
  },
  {
    path: 'discover',
    data: ['Discover', {sideBarParentTitle: 'documentation'}],
    component: DiscoverComponent
  },
  {
    path: 'schematics',
    data: ['Schematics', {sideBarParentTitle: 'documentation'}],
    component: SchematicsComponent
  },
  // hidden while themes are not implemented
  // {
  //   path: 'themes',
  //   data: ['Themes'],
  //   component: ConstructionPageComponent
  // },
  {
    path: 'components',
    children: [
      {
        path: '',
        data: ['Components'],
        component: ComponentsPageComponent
      },
      {
        path: 'accordion',
        data: ['Accordion', {moduleName: 'AccordionModule', moduleFolder: 'accordion', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/accordion').then(m => m.DemoAccordionModule)
      },
      {
        path: 'alerts',
        data: ['Alerts', {moduleName: 'AlertModule', moduleFolder: 'alert', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/alerts').then(m => m.DemoAlertsModule)
      },
      {
        path: 'buttons',
        data: ['Buttons', {moduleName: 'ButtonsModule', moduleFolder: 'buttons', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/buttons').then(m => m.DemoButtonsModule)
      },
      {
        path: 'carousel',
        data: ['Carousel', {moduleName: 'CarouselModule', moduleFolder: 'carousel', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/carousel').then(m => m.DemoCarouselModule)
      },
      {
        path: 'collapse',
        data: ['Collapse', {moduleName: 'CollapseModule', moduleFolder: 'collapse', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/collapse').then(m => m.DemoCollapseModule)
      },
      {
        path: 'datepicker',
        data: ['Datepicker', {moduleName: 'BsDatepickerModule', moduleFolder: 'datepicker', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/datepicker').then(m => m.DemoDatepickerModule)
      },
      {
        path: 'dropdowns',
        data: ['Dropdowns', {moduleName: 'BsDropdownModule', moduleFolder: 'dropdown', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/dropdown').then(m => m.DemoDropdownModule)
      },

      {
        path: 'modals',
        data: ['Modals', {moduleName: 'ModalModule', moduleFolder: 'modal', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/modal').then(m => m.DemoModalModule)
      },
      {
        path: 'pagination',
        data: ['Pagination', {moduleName: 'PaginationModule', moduleFolder: 'pagination', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/pagination').then(m => m.DemoPaginationModule)
      },
      {
        path: 'popover',
        data: ['Popover', {moduleName: 'PopoverModule', moduleFolder: 'popover', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/popover').then(m => m.DemoPopoverModule)
      },
      {
        path: 'progressbar',
        data: ['Progressbar', {moduleName: 'ProgressbarModule', moduleFolder: 'progressbar', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/progressbar').then(m => m.DemoProgressbarModule)
      },
      {
        path: 'rating',
        data: ['Rating', {moduleName: 'RatingModule', moduleFolder: 'rating', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/rating').then(m => m.DemoRatingModule)
      },
      {
        path: 'sortable',
        data: ['Sortable', {moduleName: 'SortableModule', moduleFolder: 'sortable', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/sortable').then(m => m.DemoSortableModule)
      },
      {
        path: 'tabs',
        data: ['Tabs', {moduleName: 'TabsModule', moduleFolder: 'tabs', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/tabs').then(m => m.DemoTabsModule)
      },
      {
        path: 'timepicker',
        data: ['Timepicker', {moduleName: 'TimepickerModule', moduleFolder: 'timepicker', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/timepicker').then(m => m.DemoTimepickerModule)
      },
      {
        path: 'tooltip',
        data: ['Tooltip', {moduleName: 'TooltipModule', moduleFolder: 'tooltip', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/tooltip').then(m => m.DemoTooltipModule)
      },
      {
        path: 'typeahead',
        data: ['Typeahead', {moduleName: 'TypeaheadModule', moduleFolder: 'typeahead', sideBarParentTitle: 'components', parentRoute: 'components'}],
        loadChildren: () => import('./docs/typeahead').then(m => m.DemoTypeaheadModule)
      }
    ]},
  {
    path: '**',
    redirectTo: '/'
  }
];
