"use client";

import { useAuth } from "../contexts/AuthContext";
import { useEffect, useState } from "react";
import { getCurrentUser, deleteCurrentUser } from "../services/api";
import { User } from "../types/user";
import { Authenticator } from "@aws-amplify/ui-react";
import "@aws-amplify/ui-react/styles.css";
import { amplifyConfigured } from "../config/amplify";
import { ThemeToggle } from "../components/ThemeToggle";

export default function Home() {
  const { isAuthenticated, isLoading, signOut } = useAuth();
  const [userData, setUserData] = useState<User | null>(null);
  const [apiLoading, setApiLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isAuthenticated) {
      fetchUserData();
    }
  }, [isAuthenticated]);

  const fetchUserData = async () => {
    try {
      setApiLoading(true);
      setError(null);
      const data = await getCurrentUser();
      setUserData(data);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to fetch user data",
      );
      console.error("Error fetching user data:", err);
    } finally {
      setApiLoading(false);
    }
  };

  const handleDeleteUser = async () => {
    if (
      !window.confirm(
        "Are you sure you want to delete your account? This action cannot be undone.",
      )
    ) {
      return;
    }
    try {
      setApiLoading(true);
      await deleteCurrentUser();
      await signOut();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete user");
      console.error("Error deleting user:", err);
    } finally {
      setApiLoading(false);
    }
  };

  if (isLoading) {
    return (
      <>
        <ThemeToggle />
        <div className="min-h-screen flex items-center justify-center bg-background">
          <div className="text-center">
            <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-foreground mx-auto"></div>
            <p className="mt-4 text-muted-foreground">Loading...</p>
          </div>
        </div>
      </>
    );
  }

  return (
    <>
      <ThemeToggle />
      <div className="min-h-screen bg-background">
        {!isAuthenticated ? (
          amplifyConfigured ? (
            <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
              <div className="max-w-md w-full space-y-8">
                <div className="text-center">
                  <h2 className="text-3xl font-extrabold text-foreground">
                    Welcome to Checkmate
                  </h2>
                  <p className="mt-2 text-sm text-muted-foreground">
                    Sign in to view your chess rating
                  </p>
                </div>
                <div className="mt-8">
                  <Authenticator
                    signUpAttributes={["email"]}
                    formFields={{
                      signUp: {
                        email: {
                          label: "Email",
                          placeholder: "Enter your email",
                          isRequired: true,
                        },
                        password: {
                          label: "Password",
                          placeholder: "Enter your password",
                          isRequired: true,
                        },
                        confirm_password: {
                          label: "Confirm Password",
                          placeholder: "Confirm your password",
                          isRequired: true,
                        },
                      },
                      signIn: {
                        username: {
                          label: "Email",
                          placeholder: "Enter your email",
                        },
                        password: {
                          label: "Password",
                          placeholder: "Enter your password",
                        },
                      },
                    }}
                  />
                </div>
              </div>
            </div>
          ) : (
            <div className="min-h-screen flex items-center justify-center bg-background">
              <div className="text-center">
                <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-foreground mx-auto"></div>
                <p className="mt-4 text-muted-foreground">
                  Initializing authentication...
                </p>
                <p className="mt-2 text-sm text-secondary-foreground">
                  Check console for configuration details
                </p>
              </div>
            </div>
          )
        ) : (
          <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
            <div className="max-w-md w-full space-y-8">
              <div className="text-center">
                <h2 className="text-3xl font-extrabold text-foreground">
                  Welcome back to Checkmate!
                </h2>
                {apiLoading ? (
                  <div className="mt-6">
                    <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-foreground mx-auto"></div>
                    <p className="mt-4 text-muted-foreground">
                      Loading your rating...
                    </p>
                  </div>
                ) : error ? (
                  <div className="mt-6 bg-muted border border-error/20 rounded-md p-4">
                    <div className="flex">
                      <div className="ml-3">
                        <h3 className="text-sm font-medium text-error">
                          Error loading rating
                        </h3>
                        <div className="mt-2 text-sm text-error/80">
                          {error}
                        </div>
                        <div className="mt-4">
                          <button
                            onClick={fetchUserData}
                            className="bg-muted hover:bg-muted/80 px-3 py-1 rounded text-sm text-error"
                          >
                            Try again
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                ) : userData ? (
                  <div className="mt-6 bg-input shadow rounded-lg p-6">
                    <div className="text-center">
                      <h3 className="text-lg font-medium text-foreground mb-2">
                        Your Chess Rating
                      </h3>
                      <div className="text-6xl font-bold text-primary mb-4">
                        {userData.rating}
                      </div>
                      <p className="text-sm text-muted-foreground">
                        User ID: {userData.user_id}
                      </p>
                    </div>
                  </div>
                ) : null}
              </div>
              <div className="text-center space-y-4">
                <button
                  onClick={signOut}
                  className="bg-secondary hover:bg-secondary/80 text-secondary-foreground font-bold py-2 px-4 rounded"
                >
                  Sign Out
                </button>
                <button
                  onClick={handleDeleteUser}
                  className="bg-error hover:bg-error/80 text-primary-foreground font-bold py-2 px-4 rounded"
                >
                  Delete User
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </>
  );
}
