import { mount } from "svelte";
import MainBotMiniApp from "../mini_app/main_bot/app.svelte";
import "telegram-ui/styles";

mount(MainBotMiniApp, { target: document.body });

