import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import "../lib/amplify";
import Link from "next/link";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "ChessMate",
  description: "A Chess.com clone built with Next.js and Rust",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <header className="bg-primary p-4 text-primary-foreground">
          <nav className="container mx-auto flex justify-between items-center">
            <h1 className="text-xl font-bold">ChessMate</h1>
            <div className="space-x-4">
              <Link href="/">Home</Link>
              <Link href="/play">Play</Link>
              <Link href="/profile">Profile</Link>
              <Link href="/login">Login</Link>
              <Link href="/signup">Signup</Link>
            </div>
          </nav>
        </header>
        {children}
      </body>
    </html>
  );
}
