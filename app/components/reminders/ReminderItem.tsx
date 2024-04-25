import {
  Button,
  Checkbox,
  Group,
  Slider,
  Tooltip,
} from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconInfoCircle } from "@tabler/icons-react";
import React, { useState } from "react";

interface ReminderItemProps {
  reminder: { id: number; label: string; description: string };
  enabled: boolean;
}

const ReminderItem: React.FC<ReminderItemProps> = ({ reminder, enabled }) => {
  const [isChecked, setIsChecked] = useState(false);
  const [time, setTime] = useState(30);

  const handleChange = (value: number) => setTime(value);

  const [opened, { close, open }] = useDisclosure(false);

  return (
    <Group grow mih={60}>
      <Group justify="flex-start" gap={0}>
        <Checkbox
          label={reminder.label}
          checked={isChecked && enabled}
          onChange={() => setIsChecked(!isChecked)}
          disabled={!enabled}
          color="green"
          p={0}
          m={0}
        />
        <Tooltip label={reminder.description}>
          <Button color="green" variant="transparent" size="xs"><IconInfoCircle/></Button>
        </Tooltip>
      </Group>
      <Slider
        label={`${time} min`}
        min={1}
        max={60}
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
