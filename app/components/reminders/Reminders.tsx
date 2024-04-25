"use client";
import { Paper, Stack, Title, Switch, Group, Button } from '@mantine/core';
import React, { useState } from 'react';
import ReminderList from './ReminderList';
import Settings from '../Settings/Settings';

const Reminders = () => {
  const [enabled, setEnabled] = useState(false);
  const [settingsOpened, setSettingsOpened] = useState(false);
  const [lang, setLang] = useState('en');  // Inicialmente en inglés

  const toggleEnabled = () => {
    setEnabled(!enabled);
  };

  return (
    <Paper shadow="xl" withBorder p="xl">
      <Stack>
        <Group justify="center">
          <Title>Reminders</Title>
          <Switch size="md" checked={enabled} onChange={toggleEnabled} color="green"/>
          <Button onClick={() => setSettingsOpened(true)}>Settings</Button>
        </Group>
        <ReminderList enabled={true} lang={lang} />
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
