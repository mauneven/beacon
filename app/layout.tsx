"use client"
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
  const [lang, setLang] = useState("en");

  useEffect(() => {
    const userLanguage = navigator.language.split("-")[0];
    const defaultLanguage = "en";
    const availableLanguages = ["en", "es", "fr"];
    const selectedLanguage = availableLanguages.includes(userLanguage)
      ? userLanguage
      : defaultLanguage;

    setLang(selectedLanguage);
  }, []);

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
