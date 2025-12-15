import { useState } from "react";
import type { ReactNode } from "react";
import { AuthContext } from "./authContext";
import type { AuthContextProps } from "./authContext";

export interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [token, setTokenState] = useState<string | null>(() => {
    return localStorage.getItem("auth_token");
  });

  const setToken = (t: string | null) => {
    if (t) localStorage.setItem("auth_token", t);
    else localStorage.removeItem("auth_token");
    setTokenState(t);
  };

  const contextValue: AuthContextProps = { token, setToken };

  return (
    <AuthContext.Provider value={contextValue}>{children}</AuthContext.Provider>
  );
};
