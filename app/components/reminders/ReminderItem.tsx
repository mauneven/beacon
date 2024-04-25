import React, { useState } from "react";
import { Button, Checkbox, Group, MantineProvider, Slider, Tooltip, createTheme } from "@mantine/core";
import { IconInfoCircle } from "@tabler/icons-react";

interface ReminderItemProps {
  reminder: { id: number; label: string; description: string };
  enabled: boolean;
}

const ReminderItem: React.FC<ReminderItemProps> = ({ reminder, enabled }) => {
  const [isChecked, setIsChecked] = useState(false);
  const [time, setTime] = useState(30);

  const handleChange = (value: number) => {
    setTime(value);
    if (isChecked && enabled && Notification.permission === "granted") {
      setTimeout(() => {
        new Notification(`Time to ${reminder.label}!`);
      }, value * 60000);
    }
  };

  const theme = createTheme({
    cursorType: "pointer",
  });

  return (
    <Group mih={60} justify="space-between">
      <Group justify="flex-start" gap={0}>
        <MantineProvider theme={theme}>
          <Checkbox
            label={reminder.label}
            checked={isChecked && enabled}
            onChange={() => setIsChecked(!isChecked)}
            disabled={!enabled}
            color="green"
            p={0}
            m={0}
          />
        </MantineProvider>
        <Tooltip label={reminder.description}>
          <Button color="green" variant="transparent" size="xs">
            <IconInfoCircle />
          </Button>
        </Tooltip>
      </Group>
      <Slider
        label={`${time} min`}
        min={1}
        max={60}
        w={280}
        color="green"
        value={time}
        onChange={handleChange}
        disabled={!isChecked || !enabled}
        size={"sm"}
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