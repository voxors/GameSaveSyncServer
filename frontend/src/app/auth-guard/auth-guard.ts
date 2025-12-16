import { inject, PLATFORM_ID } from '@angular/core';
import { isPlatformBrowser } from '@angular/common';
import { Router, CanActivateFn } from '@angular/router';
import { TokenService } from '../token-service/token-service';

export const authGuard: CanActivateFn = (route, state) => {
  const platform = inject(PLATFORM_ID);
  if (!isPlatformBrowser(platform)) {
    return true;
  }

  const token = inject(TokenService).getToken();
  if (!token) {
    return inject(Router).parseUrl('/login');
  }
  return true;
};
