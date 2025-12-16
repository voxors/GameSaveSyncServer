import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { catchError, map } from 'rxjs/operators';

@Injectable({ providedIn: 'root' })
export class TokenService {
  constructor(private http: HttpClient) {}
  private readonly STORAGE_KEY = 'appToken';

  setToken(token: string): void {
    sessionStorage.setItem(this.STORAGE_KEY, token);
  }

  getToken(): string | null {
    return sessionStorage.getItem(this.STORAGE_KEY);
  }

  clear(): void {
    sessionStorage.removeItem(this.STORAGE_KEY);
  }

  test(token: string): Observable<boolean> {
    const url = 'http://localhost:3000/v1/uuid';
    const headers = new HttpHeaders({
      Authorization: `Bearer ${token}`,
    });

    return this.http.get(url, { headers, responseType: 'text' }).pipe(
      map((truc) => {
        console.log(truc);
        return true;
      }),
      catchError((err) => {
        console.log(err);
        return of(false);
      }),
    );
  }
}
