import { Injectable } from '@angular/core';

@Injectable({providedIn: 'platform'})
export class ComponentApi {
  title?: string;
  anchor?: string;
  outlet: any;
  showMethods?: boolean;
}
