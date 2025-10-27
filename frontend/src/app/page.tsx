"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";

export default function Home() {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-gray-50">
      <div className="max-w-md text-center">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">
          Welcome to ChessMate
        </h1>
        <p className="text-lg text-gray-600 mb-8">
          Challenge friends, play online, and improve your chess skills.
        </p>
        <div className="space-y-4">
          <Link href="/play">
            <Button className="w-full">Play Chess</Button>
          </Link>
          <Link href="/login">
            <Button variant="outline" className="w-full">
              Login
            </Button>
          </Link>
          <Link href="/signup">
            <Button variant="outline" className="w-full">
              Sign Up
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
