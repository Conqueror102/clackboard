import type { Metadata } from "next";
import { JetBrains_Mono, Onest } from "next/font/google";
import { GyroCursor } from "@/components/GyroCursor";
import "./globals.css";

const onest = Onest({
  variable: "--font-onest",
  subsets: ["latin"],
});

const jetbrainsMono = JetBrains_Mono({
  variable: "--font-jetbrains-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Clackboard: every key, a real switch",
  description:
    "Clackboard plays a real mechanical switch sound for every key you press and release. 26 switches, free and open source for macOS, Windows and Linux.",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html
      lang="en"
      className={`${onest.variable} ${jetbrainsMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col font-sans">
        {children}
        <GyroCursor />
      </body>
    </html>
  );
}
