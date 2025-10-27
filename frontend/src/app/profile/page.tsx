"use client";

import { useEffect, useState } from "react";
import { getCurrentUser, fetchAuthSession, signOut } from "@aws-amplify/auth";
import { useRouter } from "next/navigation";
import { useStore } from "@/lib/store";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface UserStats {
  gamesPlayed: number;
  wins: number;
  losses: number;
  draws: number;
}

export default function ProfilePage() {
  const { user, setUser, clearUser } = useStore();
  const [stats, setStats] = useState<UserStats | null>(null);
  const [loading, setLoading] = useState(true);
  const router = useRouter();

  useEffect(() => {
    const loadProfile = async () => {
      try {
        const currentUser = await getCurrentUser();
        setUser(currentUser);

        // Fetch stats from backend
        const session = await fetchAuthSession();
        const token = session.tokens?.accessToken?.toString();
        const response = await fetch(
          `${process.env.NEXT_PUBLIC_API_URL}/user/stats`,
          {
            headers: {
              Authorization: `Bearer ${token}`,
            },
          },
        );
        if (response.ok) {
          const data = await response.json();
          setStats(data);
        }
      } catch (error) {
        // If not authenticated, redirect to login
        router.push("/login");
      } finally {
        setLoading(false);
      }
    };

    loadProfile();
  }, [setUser, router]);

  const handleLogout = async () => {
    // Use signOut from Amplify
    try {
      await signOut();
      clearUser();
      router.push("/");
    } catch (error) {
      console.error("Logout error:", error);
    }
  };

  if (loading) {
    return (
      <div className="flex justify-center items-center min-h-screen">
        Loading...
      </div>
    );
  }

  if (!user) {
    router.push("/login");
    return null;
  }

  return (
    <div className="flex flex-col items-center min-h-screen bg-gray-50 p-8">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle>Your Profile</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div>
              <p>
                <strong>Email:</strong> {user.attributes?.email}
              </p>
              <p>
                <strong>Username:</strong> {user.username}
              </p>
            </div>
            <div>
              <h3 className="font-semibold">Stats:</h3>
              {stats ? (
                <ul>
                  <li>Games Played: {stats.gamesPlayed}</li>
                  <li>Wins: {stats.wins}</li>
                  <li>Losses: {stats.losses}</li>
                  <li>Draws: {stats.draws}</li>
                </ul>
              ) : (
                <p>No stats available</p>
              )}
            </div>
            <Button
              onClick={handleLogout}
              variant="destructive"
              className="w-full"
            >
              Logout
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
