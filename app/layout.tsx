"use client";
import "@mantine/core/styles.css";

import { ColorSchemeScript, Container, MantineProvider } from "@mantine/core";
import { HeaderMenu } from "./components/navigation/Navbar";
import { LanguageProvider } from "./locales/LanguageContext";
import { useEffect, useState } from "react";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [lang, setLang] = useState(() => {
    if (typeof window !== "undefined") {
      const storedLang = localStorage.getItem("language");
      const userLanguage = navigator.language.split("-")[0];
      const defaultLanguage = "en";
      const availableLanguages = ["en", "es", "fr"];

      if (storedLang && availableLanguages.includes(storedLang)) {
        return storedLang;
      }

      if (availableLanguages.includes(userLanguage)) {
        return userLanguage;
      }

      return defaultLanguage;
    }

    return "en";
  });

  useEffect(() => {
    if (typeof window !== "undefined") {
      localStorage.setItem("language", lang);
    }
  }, [lang]);

  return (
    <html lang={lang}>
      <head>
        <ColorSchemeScript />
      </head>
      <body>
        <LanguageProvider lang={lang}>
          <MantineProvider>
            <HeaderMenu />
            <Container>{children}</Container>
          </MantineProvider>
        </LanguageProvider>
      </body>
    </html>
  );
}