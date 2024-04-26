import "@mantine/core/styles.css";
import { ColorSchemeScript, Container, MantineProvider } from "@mantine/core";
import { HeaderMenu } from "./components/navigation/Navbar";
import { LanguageProvider } from "./locales/LanguageContext";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
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
