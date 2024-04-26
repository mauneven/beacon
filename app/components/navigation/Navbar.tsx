"use client";

import React, { useContext } from "react";
import {
  Group,
  Burger,
  Drawer,
  ScrollArea,
  Divider,
  rem,
  Stack,
  Container,
  Button,
  Text,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import classes from "../../../public/css/navigation/HeaderMenu.module.css";
import ThemeChanger from "../theme/ThemeChanger";
import useTranslation from "@/app/useTranslation";
import { LanguageContext } from "../../locales/LanguageContext";
import { useRouter } from "next/navigation";

export function HeaderMenu() {
  const [drawerOpened, { toggle: toggleDrawer, close: closeDrawer }] =
    useDisclosure(false);
  const { language } = useContext(LanguageContext);
  const dict = useTranslation(language);
  const router = useRouter();

  return (
    <header className={classes.header}>
      <Container className={classes.inner}>
        <Group justify="space-between">
          <Burger
            opened={drawerOpened}
            onClick={toggleDrawer}
            size="sm"
            hiddenFrom="sm"
          />
          <Button variant="transparent" c={"cyan"}>
            <Text size="xl" fw={700} onClick={() => router.push("/")}>
              Beacon
            </Text>
          </Button>
        </Group>

        <Group>
          <Group ml={50} gap={5} className={classes.links} visibleFrom="sm">
            <Button
              size="sm"
              variant="light"
              color="cyan"
              onClick={() => router.push("/about")}
            >
              {dict?.reminders.about_beacon}
            </Button>{" "}
            <ThemeChanger />
          </Group>
        </Group>

        <Drawer
          opened={drawerOpened}
          onClose={closeDrawer}
          size="100%"
          padding="md"
          title="Menu"
          hiddenFrom="sm"
          zIndex={1000000}
        >
          <ScrollArea h={`calc(100vh - ${rem(80)})`} mx="-md">
            <Divider m={0} />
            <Button
              variant="light"
              color="cyan"
              onClick={() => router.push("/about")}
            >
              {dict?.reminders.about_beacon}
            </Button>
            <Divider
              m={0}
              label="Wanna change your theme?"
              labelPosition="center"
            />
            <Stack justify="center" mt={20} align="center" gap={50}>
              <ThemeChanger />
            </Stack>
          </ScrollArea>
        </Drawer>
      </Container>
    </header>
  );
}
