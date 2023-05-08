import { AfterViewInit, Component, Inject, OnDestroy, PLATFORM_ID } from "@angular/core";
import { HttpClient } from '@angular/common/http';
import { isPlatformBrowser } from '@angular/common';
import { NavigationEnd, Router } from '@angular/router';
import { Subscription } from "rxjs";

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'top-menu',
  templateUrl: './top-menu.component.html',
})
export class TopMenuComponent implements AfterViewInit, OnDestroy {
  shadowRoutes = ['/documentation', '/discover', '/schematics', '/'];
  appUrl?: string;
  appHash?: string;
  currentVersion?: string;
  isBrowser: boolean;
  initBoxShadow = false;
  isLocalhost = false;
  needPrefix = false;
  routeSubscription?: Subscription;
  previousDocs: {
    url: string;
    version: string;
    unprefixedUrl: string;
  }[] = [];

  constructor(
    @Inject(PLATFORM_ID) platformId: number,
    private http: HttpClient,
    private router: Router
  ) {
    this.isBrowser = isPlatformBrowser(platformId);
  }


  ngAfterViewInit(): void {
    if (!this.isBrowser) {
      return;
    }

    // todo: remove this sh**
    if (typeof window !== 'undefined') {
      this.isLocalhost = location.hostname === 'localhost';
      this.needPrefix = location.pathname !== '/';

      this.appUrl = location.protocol + '//' + location.hostname + (this.isLocalhost ? ':' + location.port + '/' : '/');

      this.http.get<any>('assets/json/versions.json')
        .subscribe((data: { url: string; version: string; unprefixedUrl: string }[]) => {
          this.previousDocs.push(data[0]);
          this.previousDocs = this.previousDocs
            .concat(data.reverse())
            .slice(0, -1);
        });

      this.http.get<{ version: string }>('assets/json/current-version.json')
        .subscribe((data: { version: string }) => {
          this.currentVersion = data.version;
        });
    }

    const getUrl = (router: Router) => {
      const indexOfHash = router.routerState.snapshot.url.indexOf('#');
      return indexOfHash ? router.routerState.snapshot.url : router.routerState.snapshot.url.slice(0, indexOfHash);
    };

    let _prev = getUrl(this.router);
    this.routeSubscription = this.router.events.subscribe((event: any) => {
      const _cur = getUrl(this.router);
      this.initBoxShadow = this.shadowRoutes.includes(_cur);
      if (typeof window !== 'undefined') {
        this.appHash = location.hash === '#/' ? '' : location.hash;
      }

      if (event instanceof NavigationEnd && _cur !== _prev) {
        _prev = _cur;
      }
    });
  }

  ngOnDestroy() {
    this.routeSubscription?.unsubscribe();
  }
}
