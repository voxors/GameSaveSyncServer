import { createContext, useContext } from "react";
import type { AuthContextProps } from "./authProvider";

export const AuthContext = createContext<AuthContextProps | undefined>(
  undefined,
);

export const useAuth = () => {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
};
