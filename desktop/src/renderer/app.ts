/**
 * @file app.ts
 * @description 客户端 Vue 3 启动引擎与全局服务注册层
 * @author Antigravity
 */

import { createApp } from "vue";
import App from "./App.vue";
import { ACPService } from "./services/ACPService";
import { StateService } from "./services/StateService";
import { I18nService } from "./services/I18nService";
import "./style.css";

// 实例化底层单例服务
const acpService = ACPService.getInstance();
const stateService = StateService.getInstance();
const i18nService = I18nService.getInstance();

// 连接初始化生命周期由 App.vue 的 onMounted 统一管理，此处仅负责服务实例化与挂载


// 实例化并挂载 Vue 3 应用
const app = createApp(App);

// 依赖注入全局单例，使所有子组件可以方便地 inject 使用
app.provide("acpService", acpService);
app.provide("stateService", stateService);
app.provide("i18nService", i18nService);

app.mount("#app");
