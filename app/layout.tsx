"use client"
import "@mantine/core/styles.css";
import { ColorSchemeScript, Container, MantineProvider } from "@mantine/core";
import { HeaderMenu } from "./components/navigation/Navbar";
import { useEffect, useState } from "react";
import { LanguageProvider } from "./locales/LanguageContext";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [lang, setLang] = useState('en');

  useEffect(() => {
    const storedLang = localStorage.getItem('language') ?? 'en';
    setLang(storedLang);
  }, []);

  return (
    <html lang="en">
      <head>
        <ColorSchemeScript />
      </head>
      <body>
        <LanguageProvider>
          <MantineProvider>
            <HeaderMenu />
            <Container>{children}</Container>
          </MantineProvider>
        </LanguageProvider>
      </body>
    </html>
  );
}