"use client"
import Link from "next/link";
import {
  Divider,
  Group,
  Image,
  Paper,
  Stack,
  Title,
  Text,
} from "@mantine/core";
import React, { useContext } from 'react';
import { LanguageContext } from "../locales/LanguageContext";
import useTranslation from "../useTranslation";

const About = () => {
  const { language } = useContext(LanguageContext);
  const dict = useTranslation(language);

  return (
    <Paper mt={120} mb={120} withBorder shadow="xl" p={"xl"}>
      <Group justify="space-between" grow>
        <Title order={1}>{dict?.reminders.about_page.main_title}</Title>
        <Image w={150} src={"/ico/beacon.ico"} />
      </Group>
      <Divider mt={30} mb={50} />
      <Stack>
        <Text fw={700}>{dict?.reminders.about_page.about_beacon}</Text>
        <Text>
          {dict?.reminders.about_page.about_beacon_desc_part1}
          <Link href="https://github.com/mauneven/beacon" target="_blank">GitHub</Link>
          {dict?.reminders.about_page.about_beacon_desc_part2}
          <Link href="https://github.com/elfenware/badger" target="_blank">Badger</Link>
          {dict?.reminders.about_page.about_beacon_desc_part3}
        </Text>
        <Text fw={700}>{dict?.reminders.about_page.privacy}</Text>
        <Text>{dict?.reminders.about_page.privacy_desc}</Text>
        <Text fw={700}>{dict?.reminders.about_page.license}</Text>
        <Text>{dict?.reminders.about_page.license_desc}</Text>
        <Text fw={700}>{dict?.reminders.about_page.maintainers}</Text>
        <Text>
          <Link href="https://www.mauneven.com/" target="_blank">mauneven</Link>
        </Text>
      </Stack>
    </Paper>
  );
};

export default About;
