import { Flex } from "@mantine/core";
import React from "react";
import Reminders from "./components/reminders/Reminders";

const page = () => {
  return (
    <Flex
      mih={50}
      gap="xl"
      mt={80}
      justify="center"
      align="center"
      direction="row"
      wrap="wrap"
    >
      <Reminders />
    </Flex>
  );
};

export default page;
