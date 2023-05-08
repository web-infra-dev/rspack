import { Component, OnDestroy } from "@angular/core";
import { NavigationEnd, Router, UrlSegment } from "@angular/router";
import { Subscription } from "rxjs";

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'bread-crumbs',
  templateUrl: 'breadCrumbs.component.html'
})

export class BreadCrumbsComponent implements OnDestroy{
  routeSubscription: Subscription;
  routeArray?: string[];

  constructor(
    private router: Router
  ) {
    this.routeSubscription = this.router.events.subscribe((event: any) => {
      if (event instanceof NavigationEnd) {
        this.routeArray = [];
        const tree:  UrlSegment[] = this.router.parseUrl(event.url).root.children["primary"].segments;
        tree.map(segment => {
            this.routeArray?.push(segment.path);
        });
      }
    });
  }

  navigate(index?: number) {
    if (!this.routeArray || !index && index !== 0) {
      this.router.navigate(['']);
      return;
    }

    index++;
    if (index >= this.routeArray.length) {
      return;
    }

    const urls = this.routeArray.slice(0, index);
    this.router.navigate([`/${urls.join('/')}`]);
  }

  ngOnDestroy() {
    this.routeSubscription.unsubscribe();
  }
}
