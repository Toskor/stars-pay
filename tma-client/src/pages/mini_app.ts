import { mount } from "svelte";
import MiniApp from "../mini_app/telegram_app.svelte";
import "telegram-ui/styles";

mount(MiniApp, { target: document.body });
