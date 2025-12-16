import { Routes } from '@angular/router';
import { LoginComponent } from './login-componment/login-componment';
import { authGuard } from './auth-guard/auth-guard';
import { App } from './app';

export const routes: Routes = [
  { path: 'login', component: LoginComponent },
  {
    path: '',
    canActivate: [authGuard],
    component: App,
  },
  { path: '**', redirectTo: 'login' },
];
