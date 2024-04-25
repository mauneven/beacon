"use client";
import { Paper, Stack, Title, Switch, Group } from "@mantine/core";
import React, { useState } from "react";
import ReminderList from "./ReminderList";

const Reminders = () => {
  const [enabled, setEnabled] = useState(false);

  const toggleEnabled = () => {
    setEnabled(!enabled);
  };

  return (
    <Paper shadow="xl" withBorder p="xl">
      <Stack>
        <Group justify="center">
          <Title>Reminders</Title>
          <Switch size="md" checked={enabled} onChange={toggleEnabled} color="green"/>
        </Group>
        <ReminderList enabled={true} lang="en" />
      </Stack>
    </Paper>
  );
};

export default Reminders;
