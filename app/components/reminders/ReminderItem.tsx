import React, { useState, useEffect, useRef } from 'react';
import { Button, Checkbox, Group, MantineProvider, Slider, Tooltip, createTheme } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';

interface Reminder {
  id: number;
  label: string;
  description: string;
  time: number;
  check: boolean
}

interface ReminderItemProps {
  reminder: Reminder;
  enabled: boolean;
}

const isBrowser = typeof window !== "undefined";

const ReminderItem = ({ reminder, enabled }: ReminderItemProps) => {
  const persistedState = isBrowser ? JSON.parse(localStorage.getItem(`reminder_${reminder.id}`) ?? '{}') : {};
  const [isChecked, setIsChecked] = useState(persistedState.isChecked || reminder.check);
  const [time, setTime] = useState(persistedState.time ?? reminder.time ?? 30);
  const notificationInterval = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (isBrowser) {
      localStorage.setItem(`reminder_${reminder.id}`, JSON.stringify({ isChecked, time }));
    }
  }, [isChecked, time]);

  useEffect(() => {
    if (isChecked && enabled && Notification.permission === 'granted') {
      notificationInterval.current = setInterval(() => {
        new Notification(`${reminder.description}!`);
      }, time * 60000);
    } else if (notificationInterval.current) {
      clearInterval(notificationInterval.current);
      notificationInterval.current = null;
    }

    return () => {
      if (notificationInterval.current) {
        clearInterval(notificationInterval.current);
      }
    };
  }, [isChecked, enabled, time]);

  const handleChange = (value: number) => {
    setTime(value);
  };

  const theme = createTheme({ cursorType: 'pointer' });

  return (
    <Group mih={90} justify="space-between">
      <Group justify="flex-start" gap={0}>
        <MantineProvider theme={theme}>
          <Checkbox
            label={reminder.label}
            checked={isChecked && enabled}
            onChange={() => setIsChecked(!isChecked)}
            disabled={!enabled}
            color="orange"
            p={0}
            m={0}
          />
        </MantineProvider>
        <Tooltip label={reminder.description}>
          <Button color="orange" variant="transparent" size="xs">
            <IconInfoCircle />
          </Button>
        </Tooltip>
      </Group>
      <Slider
        label={`${time} min`}
        min={1}
        max={60}
        w={380}
        color="orange"
        value={time}
        onChange={handleChange}
        disabled={!isChecked || !enabled}
        size={'sm'}
        labelAlwaysOn
        marks={[
          { value: 0, label: "0 Min" },
          { value: 30, label: "30 Min" },
          { value: 60, label: "1 Hour" },
        ]}
      />
    </Group>
  );
};

export default ReminderItem;