import React from "react";
import ReminderItem from "./ReminderItem";

const reminders = [
  { id: 1, label: "Drink Water", description: "Stay hydrated, stay sharp." },
  { id: 2, label: "Stretch Hands", description: "Give your hands a break." },
  { id: 3, label: "Stretch Legs", description: "Stretch those legs out!" },
  { id: 4, label: "Relax Eyes", description: "Rest your eyes for a moment." },
  { id: 5, label: "Sit Properly", description: "Sit up straight, feel great." },
];

interface ReminderListProps {
  enabled: boolean;
}

const ReminderList: React.FC<ReminderListProps> = ({ enabled }) => {
  return (
    <>
      {reminders.map((reminder) => (
        <ReminderItem key={reminder.id} reminder={reminder} enabled={enabled} />
      ))}
    </>
  );
};

export default ReminderList;
