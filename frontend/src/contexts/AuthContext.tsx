"use client";

import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  ReactNode,
} from "react";
import {
  signIn,
  signOut,
  getCurrentUser,
  signUp,
  confirmSignUp,
  type SignInInput,
  type SignUpInput,
} from "aws-amplify/auth";
import { Hub } from "aws-amplify/utils";

interface User {
  userId: string;
  username: string;
  email: string;
}

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  signIn: (input: SignInInput) => Promise<void>;
  signUp: (input: SignUpInput) => Promise<void>;
  confirmSignUp: (username: string, confirmationCode: string) => Promise<void>;
  signOut: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    checkAuthState();
  }, []);

  useEffect(() => {
    const hubListener = Hub.listen("auth", ({ payload }) => {
      switch (payload.event) {
        case "signedIn":
          checkAuthState();
          break;
        case "signedOut":
          setUser(null);
          break;
        default:
          break;
      }
    });

    return () => {
      hubListener();
    };
  }, []);

  const checkAuthState = async () => {
    try {
      setIsLoading(true);
      const currentUser = await getCurrentUser();
      setUser({
        userId: currentUser.userId,
        username: currentUser.username,
        email: currentUser.signInDetails?.loginId || "",
      });
    } catch (error) {
      setUser(null);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignIn = async (input: SignInInput) => {
    try {
      setIsLoading(true);
      const result = await signIn(input);
      if (result.isSignedIn) {
        await checkAuthState();
      } else {
        // Handle additional steps like MFA if needed
      }
    } catch (error) {
      console.error("Sign in error:", error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignUp = async (input: SignUpInput) => {
    try {
      setIsLoading(true);
      const result = await signUp(input);
    } catch (error) {
      console.error("Sign up error:", error);
      const err = error as Error;
      console.error("Error details:", {
        name: err.name,
        message: err.message,
        stack: err.stack,
      });
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const handleConfirmSignUp = async (
    username: string,
    confirmationCode: string,
  ) => {
    try {
      setIsLoading(true);
      await confirmSignUp({
        username,
        confirmationCode,
      });
      // After confirmation, user might need to sign in
    } catch (error) {
      console.error("Confirm sign up error:", error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignOut = async () => {
    try {
      setIsLoading(true);
      await signOut();
      setUser(null);
    } catch (error) {
      console.error("Sign out error:", error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  };

  const value: AuthContextType = {
    user,
    isAuthenticated: !!user,
    isLoading,
    signIn: handleSignIn,
    signUp: handleSignUp,
    confirmSignUp: handleConfirmSignUp,
    signOut: handleSignOut,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
}
