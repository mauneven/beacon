import "@mantine/core/styles.css";

import { ColorSchemeScript, Container, MantineProvider } from "@mantine/core";
import { HeaderMenu } from "./components/navigation/Navbar";

export const metadata = {
  title: "Beacon",
  description: "Remind yourself good things",
};

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
        <MantineProvider><HeaderMenu/><Container>{children}</Container></MantineProvider>
      </body>
    </html>
  );
}
