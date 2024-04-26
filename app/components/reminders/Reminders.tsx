"use client"

import React, { useState } from 'react';
import { Paper, Stack, Title, Switch, Group, Button } from '@mantine/core';
import ReminderList from './ReminderList';
import Settings from '../Settings/Settings';
import { IconSettings } from '@tabler/icons-react';

const Reminders = () => {
  const [enabled, setEnabled] = useState(false);
  const [settingsOpened, setSettingsOpened] = useState(false);
  const [lang, setLang] = useState('en');

  const toggleEnabled = () => {
    setEnabled(!enabled);
  };

  return (
    <Paper shadow="xl" withBorder p="xl">
      <Stack>
        <Group justify="center">
          <Title>Reminders</Title>
          <Switch size="md" checked={enabled} onChange={toggleEnabled} color="orange"/>
          <Button color="orange" variant='' p={4} onClick={() => setSettingsOpened(true)}><IconSettings/></Button>
        </Group>
        <ReminderList enabled={enabled && !settingsOpened} lang={lang} />
      </Stack>
      <Settings
        opened={settingsOpened}
        onClose={() => setSettingsOpened(false)}
        setLang={setLang}
        currentLang={lang}
      />
    </Paper>
  );
};

export default Reminders;