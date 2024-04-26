"use client"

import React, { useState, useContext } from 'react';
import { Paper, Stack, Title, Switch, Group, Button, Text } from '@mantine/core';
import ReminderList from './ReminderList';
import Settings from '../Settings/Settings';
import { IconSettings } from '@tabler/icons-react';
import useTranslation from "../../useTranslation";
import { LanguageContext } from '../../locales/LanguageContext';

const Reminders = () => {

  const [enabled, setEnabled] = useState(JSON.parse(localStorage.getItem('remindersEnabled') ?? 'true'));
  const [settingsOpened, setSettingsOpened] = useState(false);
  const { language } = useContext(LanguageContext);
  const dict = useTranslation(language);

  const toggleEnabled = () => {
    setEnabled(!enabled);
  };

  return (
    <Paper shadow="xl" withBorder p="xl">
      <Stack>
        <Group justify="space-between">
          <Stack>
          <Title>{dict?.reminders.title}</Title>
          <Text>{dict?.reminders.info}</Text>
          </Stack>
          <Group>            
          <Switch size="md" checked={enabled} onChange={toggleEnabled} color="cyan"/>
          <Button color="cyan" variant='' p={4} onClick={() => setSettingsOpened(true)}><IconSettings/></Button>
          </Group>
        </Group>
        <ReminderList enabled={enabled && !settingsOpened} />
      </Stack>
      <Settings
        opened={settingsOpened}
        onClose={() => setSettingsOpened(false)}
      />
    </Paper>
  );
};

export default Reminders;
