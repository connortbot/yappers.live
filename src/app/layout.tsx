import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  metadataBase: new URL("https://yappers.live"),
  title: "yappers.live - Real Vibes, No Hassle",
  description: "Social games for friends at the dinner table. Real Vibes, No Hassle.",
  keywords: "social games, dinner table games, party games, friends, yappers, real vibes",
  authors: [{ name: "yappers.live" }],
  openGraph: {
    title: "yappers.live - Real Vibes, No Hassle",
    description: "Social games for friends at the dinner table. Real Vibes, No Hassle.",
    images: ["/logo.png"],
    url: "https://yappers.live",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "yappers.live - Real Vibes, No Hassle",
    description: "Social games for friends at the dinner table. Real Vibes, No Hassle.",
    images: ["/logo.png"],
  },
  icons: {
    icon: "/logo.png",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Indie+Flower&family=Short+Stack&display=swap"
          rel="stylesheet"
        />
      </head>
      <body>
        {children}
      </body>
    </html>
  );
}
