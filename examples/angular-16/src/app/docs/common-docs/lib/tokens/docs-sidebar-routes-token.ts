import { InjectionToken } from "@angular/core";
import { SidebarRoutesType } from '../models/sidebar-routes.model';

export const SIDEBAR_ROUTES = new InjectionToken<SidebarRoutesType>('structured route data for sidebar');

