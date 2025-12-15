import { createContext, useContext } from "react";

export interface AuthContextProps {
  token: string | null;
  setToken: (t: string | null) => void;
}

export const AuthContext = createContext<AuthContextProps | undefined>(
  undefined,
);

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
};
