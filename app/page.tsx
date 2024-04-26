"use client"
import { Flex, Loader } from "@mantine/core";
import React, { useEffect, useState } from "react";
import Reminders from "./components/reminders/Reminders";

const Page = () => {
    const [showReminders, setShowReminders] = useState(false);

    useEffect(() => {
        setShowReminders(true);
    }, []);

    return (
        <Flex mt={100} mb={100} gap="xl" justify="center" align="center" direction="row" wrap="wrap" >
            {showReminders ? <Reminders /> : <Loader color="cyan" type="bars" />}
        </Flex>
    );
};

export default Page;