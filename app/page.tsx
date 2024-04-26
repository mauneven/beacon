import { Flex } from "@mantine/core";
import React from "react";
import Reminders from "./components/reminders/Reminders";

const page = () => {
  return (
    <Flex
      mt={100}
      mb={100}
      gap="xl"
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
