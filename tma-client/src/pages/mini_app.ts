import { mount } from "svelte";
import MiniApp from "../mini_app/stream_bot/app.svelte";
import "telegram-ui/styles";

mount(MiniApp, { target: document.body });
